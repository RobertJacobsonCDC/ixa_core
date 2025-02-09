use std::{
    any::{Any, TypeId},
    cell::{RefCell},
    collections::HashMap
};
use crate::{
    New,
    PersonId,
    error::IxaError,
    people::{Index, IndexMap, InitializationList},
    property::{Property, PropertyInfo},
    property_map::{PropertyMap, PropertyStore}
};

/// Stores all data associated to people and their properties.
pub(crate) struct PeopleData {
    // pub(super) is_initializing: bool,
    pub(crate) current_population: usize,
    /// Map from type `T: Property` to `PropertyStore`, a wrapper for `Vec<Option<T>>`
    pub(crate) properties_map: PropertyMap,
    /// Records which types have been registered with all of their dependencies in `dependency_map`
    pub(crate) registered_derived_properties: Vec<TypeId>,
    /// Maps dependencies to types that depend on them
    pub(crate) dependency_map: HashMap<TypeId, Vec<TypeId>>,
    /// This is actually a `HashMap<TypeId, IndexCore<T: Property>`
    pub(crate) property_indexes: RefCell<IndexMap>,
    /// A database of basic information about registered properties:
    ///     `PropertyInfo(Name, TypeId, IsRequired, IsDerived)`
    pub(crate) property_metadata: Vec<PropertyInfo>,
}

impl Default for PeopleData {
    fn default() -> Self {
        PeopleData {
            current_population: 0,
            properties_map: PropertyMap::new(),
            registered_derived_properties: vec![],
            dependency_map: HashMap::new(),
            property_indexes: RefCell::new(IndexMap::default()),
            property_metadata: vec![],
        }
    }
}

impl New for PeopleData {
    const new: &'static dyn Fn() -> Self = &PeopleData::default;
}

impl PeopleData {
    pub fn create_population(&mut self, size: usize) {
        self.current_population = size;
    }

    pub fn add_person(&mut self) -> PersonId {
        let person_id = PersonId(self.current_population);
        self.current_population += 1;
        person_id
    }

    pub fn get_person_property_ref<T: Property>(&self, person_id: PersonId) -> &Option<T> {
        let idx = person_id.0;
        let property_store: &PropertyStore<T> = unsafe{ self.properties_map.get_container_ref_unchecked() };

        if idx >= property_store.len() {
            &None
        } else {
            &property_store.values[idx]
        }
    }

    pub fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T> {
        let idx = person_id.0;
        let property_values: &mut PropertyStore<T> = self.properties_map.get_container_mut();

        if idx >= property_values.len() {
            property_values.values.resize_with(idx + 1, || None);
        }

        &mut property_values.values[idx]
    }

    pub fn set_property<T: Property>(&mut self, person_id: PersonId, value: T) {
        let property = self.get_person_property_mut(person_id);
        *property = Some(value);
    }

    pub fn get_index_mut<T: Property>(&mut self) -> &mut Index<T> {
        self.property_indexes
            .get_mut()
            .get_container_mut::<T>()
            // .map(|idx| {
            //     idx.downcast_mut::<Box<Index<T>>>()
            //         .unwrap()
            //         .as_mut()
            // })
    }

    pub fn get_index_ref<T: Property>(&mut self) -> Option<&Index<T>> {
        self.property_indexes
            .get_mut()
            .get_container_ref::<T>()
    }

    pub(super) fn check_initialization_list<T: InitializationList>(&self, initialization: &T)
        -> Result<(), IxaError>
    {
        for property_info in self.property_metadata.iter() {
            if property_info.is_required() && !initialization.has_property(property_info.type_id()) {
                return Err(IxaError::IxaError(format!("Missing initial value {}", property_info.name())));
            }
        }

        Ok(())
    }

}


#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::people::ContextPeopleExt;
    use super::*;

    #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    struct Age(u8);
    impl Property for Age {}

    #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    struct Name(String);
    impl Property for Name {}

    #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    enum InfectionStatus {
        I,
        R,
        S,
    }
    impl Property for InfectionStatus {}


    #[test]
    fn test_people_data() {
        let mut context = Context::new();

        context.add_person((Age(10), Name("John Smith".to_string()), InfectionStatus::I))
               .expect("Failed to add person");
    }
}
