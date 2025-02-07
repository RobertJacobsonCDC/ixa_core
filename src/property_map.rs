/*!

A map from `T: Property` to `PropertyStore` in the `AnyMap` pattern.

*/

use crate::{
    define_any_map_container,
    property::Property,
};

pub(crate) struct PropertyStore<T: Property> {
    pub is_required: bool,
    pub values: Vec<Option<T>>,
}

impl<T: Property> PropertyStore<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            is_required: false,
            values: Vec::new(),
        }
    }
    #[inline(always)]
    pub fn push(&mut self, property: T) {
        self.values.push(Some(property));
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

define_any_map_container!(
    PropertyMap, 
    PropertyStore<T: Property>, 
    PropertyStore::<T>::new(), 
    PropertyStore::push
);
