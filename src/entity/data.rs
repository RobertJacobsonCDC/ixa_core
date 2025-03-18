use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap
};
use crate::{
    New,
    EntityId,
    error::IxaError,
    entity::{Index, IndexMap, InitializationList},
    property::{Property, PropertyInfo},
    property_map::{PropertyMap, PropertyStore}
};

/// Stores all data associated to entities and their properties.
pub struct EntityData {
    /// FLag to prevent `set_property` event from being generated upon new entity creation.
    pub(super) is_initializing: bool,
    /// How many entities exist.
    pub(crate) entity_count: usize,
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

impl Default for EntityData {
    fn default() -> Self {
        EntityData {
            is_initializing: false,
            entity_count: 0,
            properties_map: PropertyMap::new(),
            registered_derived_properties: vec![],
            dependency_map: HashMap::new(),
            property_indexes: RefCell::new(IndexMap::default()),
            property_metadata: vec![],
        }
    }
}

impl New for EntityData {
    const new: &'static dyn Fn() -> Self = &EntityData::default;
}

impl EntityData {
    pub fn create_entities(&mut self, size: usize) {
        self.entity_count = size;
    }

    pub fn add_entity(&mut self) -> EntityId {
        let entity_id = EntityId(self.entity_count);
        self.entity_count += 1;
        entity_id
    }

    pub fn get_property_ref<T: Property>(&self, entity_id: EntityId) -> Option<&T> {
        
        let idx = entity_id.0;
        match self.properties_map.get_container_ref::<T>() {
            Some(property_store) if idx >= property_store.len() =>  None,

            Some(property_store) => {
                property_store.values[idx].as_ref()
            }
            
            None => None
        }
    }

    pub fn get_property_mut<T: Property>(&mut self, entity_id: EntityId) -> &mut Option<T> {
        assert!(!T::is_derived(), "Cannot set a derived property: {}", T::name());
        let idx = entity_id.0;
        let property_values: &mut PropertyStore<T> = self.properties_map.get_container_mut();

        if idx >= property_values.len() {
            property_values.values.resize_with(idx + 1, || None);
        }

        &mut property_values.values[idx]
    }

    pub fn set_property<T: Property>(&mut self, entity_id: EntityId, value: T) {
        assert!(!T::is_derived(), "Cannot set a derived property: {}", T::name());
        let property = self.get_property_mut(entity_id);
        *property = Some(value);
    }

    pub(crate) fn get_index_mut<T: Property>(&mut self) -> &mut Index<T> {
        self.property_indexes
            .get_mut()
            .get_container_mut::<T>()
    }

    pub(crate) fn get_index_ref<T: Property>(&mut self) -> Option<&Index<T>> {
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

    /// Convenience function to iterate over the current set of entities.
    /// Note that this doesn't hold a reference to EntityData, so if
    /// you change the entity count while using it, it won't notice.
    pub(super) fn entity_iterator(&self) -> Box<dyn Iterator<Item =EntityId>> {
        pub(super) struct EntityIterator {
            entity_count: usize,
            entity_id: usize,
        }

        impl Iterator for EntityIterator {
            type Item = EntityId;

            fn next(&mut self) -> Option<Self::Item> {
                let ret = if self.entity_id < self.entity_count {
                    Some(EntityId(self.entity_id))
                } else {
                    None
                };
                self.entity_id += 1;

                ret
            }
        }

        Box::new(
            EntityIterator {
                entity_count: self.entity_count,
                entity_id: 0,
            }
        )
    }

}


#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::entity::ContextEntityExt;
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
    fn test_entity_data() {
        let mut context = Context::new();

        context.add_entity((Age(10), Name("John Smith".to_string()), InfectionStatus::I))
               .expect("Failed to add person");
    }
}
