use crate::{
  type_of,
  TypeId
};
use std::{
  any::Any,
  collections::HashMap
};

#[derive(Default)]
pub struct TraitMap {
  map: HashMap<TypeId, Box<dyn Any>>,
}

impl TraitMap {
  pub fn new() -> Self {
    TraitMap {
      map: HashMap::default(),
    }
  }

  pub fn insert<T: Any>(&mut self, value: T) -> Option<Box<T>> {
    self.map
        .insert(type_of::<T>(), Box::new(value))
        .map(|boxed|
            // ToDo: Use `Any::downcast_unchecked` (nightly feature).
            // Guaranteed safe, as only a Box<T> can be a value for `type_of::<T>()`.
            unsafe { boxed.downcast().unwrap_unchecked() }
        )
  }

  pub fn get<T: Any>(&self) -> Option<&T> {
    self.map
        .get(&type_of::<T>())
        .map(|boxed|
            // ToDo: Use `Any::downcast_ref_unchecked` (nightly feature).
            // Guaranteed safe, as only a Box<T> can be a value for `type_of::<T>()`.
            unsafe { boxed.downcast_ref().unwrap_unchecked() }
        )
  }

  pub unsafe fn get_unchecked<T: Any>(&self) -> &T {
    unsafe{
      self.map
          .get(&type_of::<T>())
          .unwrap_unchecked()
          .as_ref()
          // ToDo: Use `Any::downcast_ref_unchecked` (nightly feature).
          .downcast_ref()
          // Guaranteed safe, as only a Box<T> can be a value for `type_of::<T>()`.
          .unwrap_unchecked()
    }
  }

  pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
    self.map
        .get_mut(&type_of::<T>())
        .map(|boxed|
            // ToDo: Use `Any::downcast_mut_unchecked` (nightly feature).
            // Guaranteed safe, as only a Box<T> can be a value for `type_of::<T>()`.
            unsafe { boxed.downcast_mut().unwrap_unchecked() }
        )
  }
  
  pub fn contains_key<T: Any>(&self) -> bool {
    self.map.contains_key(&type_of::<T>())
  }

  pub fn remove<T: Any>(&mut self) -> Option<Box<T>> {
    self.map
        .remove(&type_of::<T>())
        .map(|boxed|
            // ToDo: Use `Any::downcast_unchecked` (nightly feature).
            // Guaranteed safe, as only a Box<T> can be a value for `type_of::<T>()`.
            unsafe{ boxed.downcast().unwrap_unchecked() }
        )
  }

  pub fn clear(&mut self) {
    self.map.clear();
  }
}
