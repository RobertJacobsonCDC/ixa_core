use crate::{
    entity::EntityData,
    property::Property,
    type_of,
    EntityId,
    TypeId
};
use seq_macro::seq;

/// A trait that contains the initialization values for a
/// new entity. Do not use this directly, but instead use
/// the tuple syntax.
pub trait InitializationList {
    fn has_property(&self, t: TypeId) -> bool;
    fn set_properties(self, entity_data: &mut EntityData, entity_id: EntityId);
}

// Implement the query version with 0 and 1 parameters
impl InitializationList for () {
    fn has_property(&self, _: TypeId) -> bool {
        false
    }
    fn set_properties(self, _entity_data: &mut EntityData, _entity_id: EntityId) {}
}

impl<T1: Property> InitializationList for T1 {
    fn has_property(&self, t: TypeId) -> bool {
        t == type_of::<T1>()
    }

    fn set_properties(self, entity_data: &mut EntityData, entity_id: EntityId) {
        entity_data.set_property::<T1>(entity_id, self);
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

                fn set_properties(self, entity_data: &mut EntityData, entity_id: EntityId)  {
                    #(
                       entity_data.set_property(entity_id, self.N );
                    )*
                }
            }
        });
    }
}

seq!(Z in 1..20 {
    impl_initialization_list!(Z);
});
