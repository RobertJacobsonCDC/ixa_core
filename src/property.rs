use std::any::TypeId;
use std::task::Context;
use crate::PersonId;

pub trait Property: Clone + 'static {

  #[must_use]
  fn is_derived() -> bool {
    false
  }

  #[must_use]
  fn name() -> &'static str {
    std::any::type_name::<Self>()
  }

}

pub trait DerivedProperty: Property {

  #[must_use]
  fn dependencies() -> Vec<TypeId>;

  #[must_use]
  fn compute(context: &Context, person_id: PersonId) -> Self;

}


/// Any type that is `Clone + 'static`
#[derive(Debug, Copy, Clone)]
pub struct DerivedPropertyName(bool);
/*
// Synthesized impls for `Property` and `DerivedProperty`
impl Property for DerivedPropertyName {

  fn is_derived() -> bool { true }

  fn name() -> &'static str {
    stringify!( DerivedPropertyName )
  }
}

impl DerivedProperty for DerivedPropertyName {
  fn dependencies() -> Vec<TypeId> {
    vec![crate::type_of::<PersonProperty1>()]
  }
  fn compute(context: &Context, person_id: PersonId) -> Self {
    #[allow(unused_imports)]
    use crate::ContextGlobalPropertiesExt;
    #[allow(unused_parens)]
    let (person_prop, global_prop, ) = (
      context.get_person_property::<PersonProperty1>(person_id),
      *context.get_global_property_value::<GlobalProperty1>()
              .expect(&format!("Global property {} not initialized", stringify!( GlobalProperty1 ))),
    );
    (|person_prop, global_prop| { person_prop >= global_prop }
    )(person_prop, global_prop)
  }
}
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
            fn is_derived() -> bool { true }

            fn name() -> &'static str { stringify!($derived_property) }
        }

        impl $crate::DerivedProperty for $derived_property {
          fn dependencies() -> Vec<std::any::TypeId> {
                vec![$(crate::type_of::<$dependency>()),+]
            }

            fn compute(context: &$crate::context::Context, person_id: $crate::people::PersonId) -> Self {
                #[allow(unused_imports)]
                use $crate::ContextGlobalPropertiesExt;
                #[allow(unused_parens)]
                let ($($param,)*) = (
                    $(context.get_person_property_unchecked::<$dependency>(person_id)),*,
                    $(
                        *context.get_global_property_value::<$global_dependency>()
                            .expect(&format!("Global property {} not initialized", stringify!($global_dependency))),
                    ),*
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

// define_derived_property!(DerivedPropertyName, [PersonProperty1], [GlobalProperty1], |person_prop, global_prop| { DerivedPropertyName(person_prop >= global_prop) });
