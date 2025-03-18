use crate::{context::Context, error::IxaError, entity::{
    Index,
    IndexValue,
    InitializationList,
    EntityData,
    Query
}, EntityId, property::{
    Property
}, type_of, HashMap};

pub trait ContextEntityExt {
    fn get_entity_count(&self) -> usize;
    fn add_entity<T: InitializationList>(&mut self, properties: T) -> Result<EntityId, IxaError>;

    fn get_property<T: Property>(&mut self, entity_id: EntityId) -> Option<T>;
    fn get_property_mut<T: Property>(&mut self, entity_id: EntityId) -> &mut Option<T>;
    fn get_property_or_default<T: Property>(
        &mut self,
        entity_id: EntityId,
        default: T,
    ) -> &mut T;

    fn set_property<T: Property>(&mut self, entity_id: EntityId, value: T);

    fn query_entities<T: Query>(&mut self, q: T) -> Vec<EntityId>;

    /// Get the count of all entities matching a given set of criteria.
    ///
    /// [`Context::query_entity_count()`] takes any type that implements [Query],
    /// but instead of implementing query yourself it is best
    /// to use the automatic syntax that implements [Query] for
    /// a tuple of pairs of (property, value), like so:
    /// `context.query_entities(((Age, 30), (Gender, Female)))`.
    ///
    /// This is intended to be slightly faster than [`Context::query_entities()`]
    /// because it does not need to allocate a list. We haven't actually
    /// measured it, so the difference may be modest if any.
    fn query_entity_count<T: Query>(&mut self, q: T) -> usize;

    /// Determine whether an entity matches a given expression.
    ///
    /// The syntax here is the same as with [`Context::query_entities()`].
    fn match_entity<T: Query>(&mut self, person_id: EntityId, q: T) -> bool;

}

impl ContextEntityExt for Context {
    fn get_entity_count(&self) -> usize {
        match self.get_data_container::<EntityData>() {
            None => 0,
            Some(entity_data) => entity_data.entity_count,
        }
    }

    /// Adds a new entity with the given list of properties.
    fn add_entity<T: InitializationList>(&mut self, properties: T) -> Result<EntityId, IxaError> {
        let entity_data = self.get_data_container_mut::<EntityData>();
        entity_data.check_initialization_list(&properties)?;

        let entity_id = entity_data.add_entity();

        // Initialize the properties. We set |is_initializing| to prevent
        // set_property() from generating an event.
        entity_data.is_initializing = true;
        properties.set_properties(entity_data, entity_id);
        entity_data.is_initializing = false;

        Ok(entity_id)
    }

    /// Gets a copy of the value of the property for the given entity.
    fn get_property<T: Property>(&mut self, entity_id: EntityId) -> Option<T> {
        T::register(self);
        T::compute(self, entity_id)
    }

    /// Gets a mutable reference to the value of the property for the given entity.
    fn get_property_mut<T: Property>(&mut self, entity_id: EntityId) -> &mut Option<T> {
        assert!(!T::is_derived());
        T::register(self);
        self.get_data_container_mut::<EntityData>()
            .get_property_mut(entity_id)
    }

    /// Gets a mutable reference to the value of the property for the given entity if it
    /// exists, or else sets the property to the default value and returns that.
    // ToDo: Does not emit event (or respect `PeopleData::is_initializing`)
    fn get_property_or_default<T: Property>(
        &mut self,
        entity_id: EntityId,
        default: T,
    ) -> &mut T {
        let property: &mut Option<T> = self
            .get_data_container_mut::<EntityData>()
            .get_property_mut(entity_id);

        match property {
            Some(value) => value,

            None => {
                *property = Some(default);
                property.as_mut().unwrap()
            }
        }
    }

    fn set_property<T: Property>(&mut self, entity_id: EntityId, value: T) {
        let property: &mut Option<T> = self
            .get_data_container_mut::<EntityData>()
            .get_property_mut(entity_id);
        *property = Some(value);
    }

    fn query_entities<T: Query>(&mut self, query: T) -> Vec<EntityId> {
        query.setup(self);

        let mut result = Vec::new();
        query.execute_query(
            self,
            |entity| {
                result.push(entity);
            }
        );

        result
    }

    fn query_entity_count<T: Query>(&mut self, q: T) -> usize {
        T::setup(&q, self);
        let mut count: usize = 0;
        q.execute_query(self,|_person| {
            count += 1;
        } );

        count
    }

    fn match_entity<T: Query>(&mut self, entity_id: EntityId, q: T) -> bool {
        q.match_entity(self, entity_id)
    }

}

pub(crate) trait ContextEntityExtInternal {
    /// Create the index for the given property. Note that this does not populate the index. That happens lazily.
    fn index_property<T: Property>(&mut self);
    /// Reports whether the property has already been registered for this context.
    fn is_registered<T: Property>(&mut self) -> bool;
    fn register_indexer<T: Property>(&mut self);
    fn add_to_index_maybe<T: Property>(&mut self, entity_id: EntityId);
    fn remove_from_index_maybe<T: Property>(&mut self, entity_id: EntityId);
    /// Registers the property with all of its dependencies and then registers an index for the type.
    fn register_derived_property<T: Property>(&mut self);
    fn register_nonderived_property<T: Property>(&mut self);
    /// A version of `get_property` that doesn't need a mutable context. This can only be called from context in which
    /// you know `Property::register` has already been called.
    fn get_property_internal<T: Property>(&self, entity_id: EntityId) -> Option<T>;
}

impl ContextEntityExtInternal for Context {
    /// Create the index for the given property. Note that this does not populate the index. That happens lazily.
    fn index_property<T: Property>(&mut self) {
        T::register(self);

        let data_container = self.get_data_container_mut::<EntityData>();
        let index = data_container.get_index_mut::<T>();
        if index.lookup.is_none() {
            index.lookup = Some(HashMap::default());
        }
    }

    /// Reports whether the property has already been registered for this context.
    fn is_registered<T: Property>(&mut self) -> bool {
        let data_container = self.get_data_container_mut::<EntityData>();
        data_container.registered_derived_properties.contains(&type_of::<T>())
    }

    fn register_indexer<T: Property>(&mut self) {
        let property_indexes = self
            .get_data_container_mut::<EntityData>()
            .property_indexes
            .get_mut();
        let type_id = type_of::<T>();

        // This method should only be called during initial Property registration.
        assert!(!property_indexes.contains_key(&type_id));
        property_indexes.insert(Index::<T>::new());
    }

    fn add_to_index_maybe<T: Property>(&mut self, entity_id: EntityId) {
        let value = self.get_property_internal::<T>(entity_id).clone();
        let index_value = IndexValue::new(&value);
        let entity_data = self.get_data_container_mut::<EntityData>();

        let index = entity_data.get_index_mut::<T>();
        if index.lookup.is_some() {
            index.insert((entity_id, index_value));
        }
    }

    fn remove_from_index_maybe<T: Property>(&mut self, entity_id: EntityId) {
        let value = self.get_property_internal::<T>(entity_id).clone();
        let index_value = IndexValue::new(&value);
        let entity_data = self.get_data_container_mut::<EntityData>();

        let index = entity_data.get_index_mut::<T>();
        if let Some(lookup) = &mut index.lookup {
            if let Some(index_set) = lookup.get_mut(&index_value) {
                index_set.remove(&entity_id);
                // Clean up the entry if there are no entities
                if index_set.is_empty() {
                    lookup.remove(&index_value);
                }
            }
        }
    }

    /// Registers the type with all of its dependencies and then registers an index for the type.
    fn register_derived_property<T: Property>(&mut self) {
        let entity_data = self.get_data_container_mut::<EntityData>();
        let type_id = type_of::<T>();

        // This method should only be called during initial Property registration.
        assert!(!entity_data.property_indexes.borrow().contains_key(&type_id));

        let mut dependencies = vec![];
        T::collect_dependencies(&mut dependencies);
        for dependency in dependencies {
            let derived_prop_list = entity_data.dependency_map.entry(dependency).or_default();
            derived_prop_list.push(type_id);
        }

        // Also do everything that needs to be done for nonderived properties
        self.register_nonderived_property::<T>();
    }

    fn register_nonderived_property<T: Property>(&mut self) {
        let entity_data = self.get_data_container_mut::<EntityData>();
        let property_info =T::property_info();

        entity_data
            .registered_derived_properties
            .push(property_info.type_id());
        entity_data
            .property_metadata
            .push(property_info);

        self.register_indexer::<T>();
    }

    fn get_property_internal<T: Property>(&self, entity_id: EntityId) -> Option<T> {
        T::compute(self, entity_id)
    }
}
