#![allow(dead_code)]

pub mod any_map;
mod context;
mod new_trait;
mod entity;
mod property;
mod property_map;
mod error;
mod random;
mod hashing;
pub mod log;
mod trait_map;
mod global_properties;

// Re-exports
pub use rand;
pub use paste;
pub use ctor;

// All modules import `crate::TypeId` in case we want to change the underlying type of `TypeId`.
pub use std::any::TypeId;
// pub(crate) use typeid::of as type_of;
pub use new_trait::New;

pub use context::Context;
pub use error::IxaError;
pub use entity::ContextEntityExt;
pub use property::Property;
pub use random::{ContextRandomExt, RngId};
pub use log::{debug, error, info, trace, warn};
pub use hashing::{HashMap, HashMapExt, HashSet, HashSetExt};

// Replace with `typeid::of as type_of` if necessary.
#[inline(always)]
pub fn type_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EntityId(pub(crate) usize);

