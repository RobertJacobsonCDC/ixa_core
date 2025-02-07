use std::cell::Ref;
use std::collections::HashSet;
use typeid::of as type_of;
use crate::{
    IxaError,
    PersonId,
    TypeId,
    context::Context,
    index::{Index, IndexValue},
    init_list::InitializationList,
    people::people_data::PeopleData,
    property::{DerivedProperty, Property},
    query::Query,
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
    
    fn query_people<T: Query>(&self, q: T) -> Vec<PersonId>;
    
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

    fn register_property<T: Property>(&mut self) {
        T::register(self);
    }

}

pub(crate) trait ContextPeopleExtInternal {
    fn register_indexer<T: Property>(&mut self);
    fn add_to_index_maybe<T: Property>(&mut self, person_id: PersonId, property: T);
    fn remove_from_index_maybe<T: Property>(&mut self, person_id: PersonId);
    fn query_people_internal(
        &mut self,
        accumulator: impl FnMut(PersonId),
        property_hashes: Vec<(TypeId, IndexValue)>,
    );
    /// Registers the type with all of its dependencies and then registers
    fn register_derived_property<T: DerivedProperty>(&mut self);
    fn register_nonderived_property<T: Property>(&mut self);
}

impl ContextPeopleExtInternal for Context {
    fn register_indexer<T: Property>(&mut self) {
        let people_data = self.get_data_container_mut::<PeopleData>();
        let type_id = type_of::<T>();

        // This method should only be called during initial Property registration.
        assert!(!people_data.property_indexes.contains_key(&type_id));
        people_data
            .property_indexes
            .insert(type_id, Box::new(Index::<T>::new()));
    }

    /// Executes the query, accumulating the results with `accumulator`.
    fn query_people_internal(&mut self, accumulator: impl FnMut(PersonId), property_hashes: Vec<(TypeId, IndexValue)>) {
        let mut indexes = Vec::<Ref<HashSet<PersonId>>>::new();
        let mut unindexed = Vec::<(TypeId, IndexValue)>::new();
        let data_container = self.get_data_container_mut::<PeopleData>();

        // 1. Walk through each property and update the indexes.
        for (t, _) in &property_hashes {
            let mut index = data_container.get_index_ref_mut(*t).unwrap();
            let methods = data_container.get_methods(*t);
            index.index_unindexed_people(self, &methods);
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
        let type_id = type_of::<T>();

        people_data
            .people_types
            .insert(T::name().to_string(), type_id);
        people_data
            .registered_derived_properties
            .push(type_id);
        
        self.register_indexer::<T>();
    }
}
