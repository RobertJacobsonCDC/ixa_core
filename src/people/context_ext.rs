use std::{
    cell::Ref,
    collections::HashSet
};
use crate::{
    context::Context,
    error::IxaError,
    people::{
        Index,
        IndexValue,
        InitializationList,
        PeopleData,
        Query
    },
    PersonId,
    property::{
        DerivedProperty, 
        Property
    },
    TypeId,
    type_of,
};

pub trait ContextPeopleExt {
    fn get_current_population(&self) -> usize;
    fn add_person<T: InitializationList>(&mut self, properties: T) -> Result<PersonId, IxaError>;

    fn get_person_property<T: Property>(&self, person_id: PersonId) -> &Option<T>;
    fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T>;
    fn get_person_property_or_default<T: Property>(
        &mut self,
        person_id: PersonId,
        default: T,
    ) -> &mut T;

    fn set_person_property<T: Property>(&mut self, person_id: PersonId, value: T);

    fn query_people<T: Query>(&mut self, q: T) -> Vec<PersonId>;

    /// Registers the type with `PeopleData`
    fn register_property<T: Property>(&mut self);
}

impl ContextPeopleExt for Context {
    fn get_current_population(&self) -> usize {
        match self.get_data_container::<PeopleData>() {
            None => 0,
            Some(people_data) => people_data.current_population,
        }
    }

    /// Adds a new person with the given list of properties.
    fn add_person<T: InitializationList>(&mut self, properties: T) -> Result<PersonId, IxaError> {
        let people_data = self.get_data_container_mut::<PeopleData>();
        people_data.check_initialization_list(&properties)?;

        let person_id = people_data.add_person();

        // Initialize the properties. We set |is_initializing| to prevent
        // set_property() from generating an event.
        // people_data.is_initializing = true;
        properties.set_properties(people_data, person_id);
        // people_data.is_initializing = false;

        Ok(person_id)
    }

    /// Gets a copy of the value of the property for the given person.
    fn get_person_property<T: Property>(&self, person_id: PersonId) -> &Option<T> {
        self.get_data_container::<PeopleData>()
            .unwrap()
            .get_person_property_ref(person_id)
    }

    /// Gets a copy of the value of the property for the given person.
    fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T> {
        self.get_data_container_mut::<PeopleData>()
            .get_person_property_mut(person_id)
    }

    /// Gets a copy of the value of the property for the given person if it
    /// exists, or else sets the property to the default value and returns that.
    // ToDo: Does not emit event (or respect `PeopleData::is_initializing`)
    fn get_person_property_or_default<T: Property>(
        &mut self,
        person_id: PersonId,
        default: T,
    ) -> &mut T {
        let property: &mut Option<T> = self
            .get_data_container_mut::<PeopleData>()
            .get_person_property_mut(person_id);

        match property {
            Some(value) => value,

            None => {
                *property = Some(default);
                property.as_mut().unwrap()
            }
        }
    }

    fn set_person_property<T: Property>(&mut self, person_id: PersonId, value: T) {
        let property: &mut Option<T> = self
            .get_data_container_mut::<PeopleData>()
            .get_person_property_mut(person_id);
        *property = Some(value);
    }

    fn query_people<T: Query>(&mut self, q: T) -> Vec<PersonId> {
        T::setup(&q, self);
        let mut result = Vec::new();
        self.query_people_internal(
            |person| {
                result.push(person);
            },
            &q,
        );
        result
    }

    fn register_property<T: Property>(&mut self) {
        T::register(self);
    }

}

pub(crate) trait ContextPeopleExtInternal {
    fn register_indexer<T: Property>(&mut self);
    fn add_to_index_maybe<T: Property>(&mut self, person_id: PersonId, property: T);
    fn remove_from_index_maybe<T: Property>(&mut self, person_id: PersonId);
    fn query_people_internal<Q: Query>(&mut self, accumulator: impl FnMut(PersonId), query: Q);
    /// Registers the type with all of its dependencies and then registers
    fn register_derived_property<T: DerivedProperty>(&mut self);
    fn register_nonderived_property<T: Property>(&mut self);
}

impl ContextPeopleExtInternal for Context {
    fn register_indexer<T: Property>(&mut self) {
        let property_indexes = self.get_data_container_mut::<PeopleData>().property_indexes.get_mut();
        let type_id = type_of::<T>();

        // This method should only be called during initial Property registration.
        assert!(!property_indexes.contains_key(&type_id));
        property_indexes.insert(Index::<T>::new());
    }

    /// Executes the query, accumulating the results with `accumulator`.
    fn query_people_internal<Q: Query>(&mut self, accumulator: impl FnMut(PersonId), query: Q) {
        let mut indexes = Vec::<Ref<HashSet<PersonId>>>::new();
        let mut unindexed = Vec::<(TypeId, IndexValue)>::new();

        // 1. Refresh the indexes for each property in the query.
        query.refresh_indexes(self);
        
        // 2. Collect the index entry corresponding to the value.
        // let property_hashes = query.get_query();
        // let people_data = self.get_data_container::<PeopleData>();
        
        for (t, hash) in property_hashes {
            let index = people_data.get_index_ref(t).unwrap();
            if let Ok(lookup) = Ref::filter_map(index, |x| x.lookup.as_ref()) {
                if let Ok(matching_people) =
                    Ref::filter_map(lookup, |x| x.get(&hash).map(|entry| &entry.1))
                {
                    indexes.push(matching_people);
                } else {
                    // This is empty and so the intersection will
                    // also be empty.
                    return;
                }
            } else {
                // No index, so we'll get to this after.
                unindexed.push((t, hash));
            }
        }

    }

    /// Registers the type with all of its dependencies and then registers an index for the type.
    fn register_derived_property<T: DerivedProperty>(&mut self) {
        let people_data = self.get_data_container_mut::<PeopleData>();
        let type_id = type_of::<T>();

        // This method should only be called during initial Property registration.
        assert!(!people_data.property_indexes.contains_key(&type_id));

        let mut dependencies = vec![];
        T::collect_dependencies(&mut dependencies);
        for dependency in dependencies {
            let derived_prop_list = people_data.dependency_map.entry(dependency).or_default();
            derived_prop_list.push(type_id);
        }

        // Also do everything that needs to be done for nonderived properties
        self.register_nonderived_property::<T>();
    }

    fn register_nonderived_property<T: Property>(&mut self) {
        let people_data = self.get_data_container_mut::<PeopleData>();
        let property_info =T::property_info();

        people_data
            .registered_derived_properties
            .push(property_info.type_id());
        people_data
            .property_metadata
            .push(property_info);

        self.register_indexer::<T>();
    }

}
