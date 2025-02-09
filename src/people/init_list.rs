use crate::{
    property::Property,
    people::PeopleData,
    PersonId,
    type_of,
    TypeId
};
use seq_macro::seq;

/// A trait that contains the initialization values for a
/// new person. Do not use this directly, but instead use
/// the tuple syntax.
pub trait InitializationList {
    fn has_property(&self, t: TypeId) -> bool;
    fn set_properties(self, people_data: &mut PeopleData, person_id: PersonId);
}

// Implement the query version with 0 and 1 parameters
impl InitializationList for () {
    fn has_property(&self, _: TypeId) -> bool {
        false
    }
    fn set_properties(self, _people_data: &mut PeopleData, _person_id: PersonId) {}
}

impl<T1: Property> InitializationList for T1 {
    fn has_property(&self, t: TypeId) -> bool {
        t == type_of::<T1>()
    }

    fn set_properties(self, people_data: &mut PeopleData, person_id: PersonId) {
        people_data.set_property::<T1>(person_id, self);
    }
}

// Implement the versions with 1..20 parameters.
macro_rules! impl_initialization_list {
    ($ct:expr) => {
        seq!(N in 0..$ct {
            impl<
                #(
                    T~N : Property,
                )*
            > InitializationList for (
                #(
                    T~N,
                )*
            )
            {
                fn has_property(&self, t: TypeId) -> bool {
                    #(
                        if t == type_of::<T~N>() { return true; }
                    )*
                    return false
                }

                fn set_properties(self, people_data: &mut PeopleData, person_id: PersonId)  {
                    #(
                       people_data.set_property(person_id, self.N );
                    )*
                }
            }
        });
    }
}

seq!(Z in 1..20 {
    impl_initialization_list!(Z);
});
