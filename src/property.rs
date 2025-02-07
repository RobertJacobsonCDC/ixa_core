use crate::{
    context::Context,
    PersonId,
    people::ContextPeopleExt,
};
use std::{
    any::TypeId,
    fmt::Debug,
    hash::Hash
};

pub trait Property: Clone + Debug + PartialEq + Hash + 'static {
    // #[must_use]
    // fn is_derived() -> bool {
    //     false
    // }

    #[must_use]
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Overridden by `DerivedProperty`s, because they also need to register dependencies.
    fn register(context: &mut Context) {
        context.register_nonderived_property::<Self>();
    }

    /// Adds all nonderived dependencies of `Self` to `dependencies`, ***including `Self`***
    /// if `Self` is nonderived.
    fn collect_dependencies(dependencies: &mut Vec<TypeId>){
        dependencies.push(crate::type_of::<Self>());
    }
}

pub trait DerivedProperty: Property {
    /// Computes this property value.
    // ToDo: This could be implemented for all `Property`s by just looking up the value of the property for
    //       nonderived properties.
    #[must_use]
    fn compute(context: &Context, person_id: PersonId) -> Self;
}

/*
//How `define_derived_property!` implements `DerivedProperty`.
/// Any type that is `Clone + 'static`
#[derive(Debug, Copy, Clone)]
pub struct DerivedPropertyName(bool);

// define_derived_property!(DerivedPropertyName, [PersonProperty1, PersonProperty2], [GlobalProperty1, GlobalProperty2], |pprop1, pprop2, gprop1, gprop2| { DerivedPropertyName(pprop1.0 >= gprop2.0) });
*/

/// Defines a derived person property with the following parameters:
/// * `$person_property`: The property type
/// * `[$($dependency),+]`: A list of person properties the derived property depends on
/// * `[$($dependency),*]`: A list of global properties the derived property depends on (optional)
/// * $calculate: A closure that takes the values of each dependency and returns the derived value
#[macro_export]
macro_rules! define_derived_property {
    (
        $derived_property:ty,
        [$($dependency:ident),*],
        [$($global_dependency:ident),*],
        |$($param:ident),+| $derive_fn:expr
    ) => {
        impl $crate::Property for $derived_property {
            fn name() -> &'static str {
                stringify!($derived_property)
            }

            fn register(context: &$crate::Context) {
                use $crate::people::ContextPeopleExt;

                context.register_derived_property::<$derived_property>();
            }

            fn collect_dependencies(dependencies: &mut Vec<std::any::TypeId>) {
                $(
                    $dependency::collect_dependencies(dependencies);
                )*
            }
        }

        impl $crate::DerivedProperty for $derived_property {
            fn compute(context: &$crate::context::Context, person_id: $crate::PersonId) -> Self {
                #[allow(unused_imports)]
                use $crate::global_properties::ContextGlobalPropertiesExt;
                #[allow(unused_parens)]
                let ($($param,)*) = (
                    $(context.get_person_property::<$dependency>(person_id).unwrap()),*,
                    $(
                        *context.get_global_property_value::<$global_dependency>()
                            .unwrap_or_else(|| panic!(
                                "Global property {} not initialized",
                                stringify!($global_dependency)
                            )),
                    )*
                );

                (|$($param),+| $derive_fn)($($param),+)
            }
        }
    };

    (
        $derived_property:ty,
        [$($dependency:ident),*],
        |$($param:ident),+| $derive_fn:expr
    ) => {
        define_derived_property!(
            $derived_property,
            [$($dependency),*],
            [],
            |$($param),+| $derive_fn
        );
    };
}
pub use define_derived_property;
