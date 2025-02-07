pub mod any_map;
mod context;
mod init_list;
mod new_trait;
mod people;
mod property;
mod query;
mod property_map;
mod error;

// All modules import `crate::TypeId` in case we want to change the underlying type of `TypeId`.
pub(crate) use std::any::TypeId;
pub use new_trait::New;
// pub(crate) use typeid::of as type_of;


// Replace with `typeid::of as type_of` if necessary.
#[inline(always)]
pub fn type_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PersonId(pub(crate) usize);

