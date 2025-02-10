#![allow(dead_code)]

mod any_map;
mod context;
mod new_trait;
mod people;
mod property;
mod property_map;
mod error;
mod random;

// All modules import `crate::TypeId` in case we want to change the underlying type of `TypeId`.
pub(crate) use std::any::TypeId;
// pub(crate) use typeid::of as type_of;
pub use new_trait::New;

pub use context::Context;
pub use error::IxaError;
pub use people::ContextPeopleExt;
pub use property::Property;
pub use random::{ContextRandomExt, RngId, define_rng};


// Replace with `typeid::of as type_of` if necessary.
#[inline(always)]
pub fn type_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PersonId(pub(crate) usize);

