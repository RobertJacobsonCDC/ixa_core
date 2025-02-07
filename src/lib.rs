pub(crate) use typeid::of as type_of;
pub(crate) use std::any::TypeId;

mod property;
mod context;
mod new_trait;
mod people;
mod any_map;
mod init_list;
mod index;
mod query;

pub use property::{Property, DerivedProperty, define_derived_property};

// // Replace with `typeid::of as type_of` if necessary.
// pub fn type_of<T: 'static>() -> TypeId {
//     TypeId::of::<T>()
// }

pub use new_trait::New;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PersonId(pub(crate) usize);

// stubs
pub trait InitializationList{}
pub type IxaError = ();


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
