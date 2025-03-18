use crate::{
    context::Context,
    EntityId,
    entity::ContextEntityExtInternal,
    TypeId,
    type_of,
};
use std::{
    any::type_name,
    fmt::Debug,
    hash::Hash,
};
use crate::entity::EntityData;

/// Basic metadata about a property, a record in a property metadata database:
///     `(Name, TypeId, IsRequired, IsDerived)`
pub struct PropertyInfo(pub String, pub TypeId, pub bool, pub bool);
impl PropertyInfo {
    #[must_use]
    #[inline(always)]
    pub fn name(&self) -> &str {
        self.0.as_str()
    }

    #[must_use]
    #[inline(always)]
    pub fn type_id(&self) -> TypeId {
        self.1
    }

    #[must_use]
    #[inline(always)]
    pub fn is_required(&self) -> bool {
        self.2
    }

    #[must_use]
    #[inline(always)]
    pub fn is_derived(&self) -> bool {
        self.3
    }
}

pub trait Property: Clone + Debug + PartialEq + Hash + 'static {
    #[must_use]
    fn is_derived() -> bool {
        false
    }

    #[must_use]
    #[inline]
    fn name() -> &'static str {
        type_name::<Self>()
    }

    #[must_use]
    #[inline]
    fn is_required() -> bool {
        false
    }

    /// Overridden by `DerivedProperty`s, because they also need to register dependencies.
    #[inline]
    fn register(context: &mut Context) {
        if !context.is_registered::<Self>(){
            context.register_nonderived_property::<Self>();
        }
    }

    /// Adds all nonderived dependencies of `Self` to `dependencies`, ***including `Self`***
    /// if `Self` is nonderived.
    #[inline]
    fn collect_dependencies(dependencies: &mut Vec<TypeId>){
        dependencies.push(type_of::<Self>());
    }

    #[must_use]
    #[inline]
    fn property_info() -> PropertyInfo {
        PropertyInfo(Self::name().to_string(), type_of::<Self>(), Self::is_required(), false)
    }

    #[must_use]
    fn compute(context: &Context, entity_id: EntityId) -> Option<Self> {
        context.get_data_container::<EntityData>()
               .unwrap()
               .get_property_ref(entity_id).cloned()
    }
}

/*
//How `define_derived_property!` implements `DerivedProperty`.
/// Any type that is `Clone + 'static`
#[derive(Debug, Copy, Clone)]
pub struct DerivedPropertyName(bool);

define_derived_property!(
    DerivedPropertyName,
    [PersonProperty1, PersonProperty2],
    [GlobalProperty1, GlobalProperty2],
    |pprop1, pprop2, gprop1, gprop2| {
        DerivedPropertyName(pprop1.0 >= gprop2.0)
    }
);
*/

/// Defines a derived person property with the following parameters:
/// * `$person_property`: The property type
/// * `[$($dependency),+]`: A list of person properties the derived property depends on
/// * `[$($dependency),*]`: A list of global properties the derived property depends on (optional)
/// * $calculate: A closure that takes the values of each dependency and returns the derived value
#[macro_export]
macro_rules! define_derived_property {
    (
        $derived_property:ident,
        [$($dependency:ident),*],
        [$($global_dependency:ident),*],
        |$($param:ident),+| $derive_fn:expr
    ) => {
        impl $crate::Property for $derived_property {
            fn is_derived() -> bool {
                true 
            }
            
            fn name() -> &'static str {
                stringify!($derived_property)
            }

            fn register(context: &mut $crate::Context) {
                use $crate::entity::ContextEntityExtInternal;
                if !context.is_registered::<Self>(){
                    context.register_derived_property::<$derived_property>();
                }
            }

            fn collect_dependencies(dependencies: &mut Vec<std::any::TypeId>) {
                $(
                    $dependency::collect_dependencies(dependencies);
                )*
            }

            fn property_info() -> $crate::property::PropertyInfo {
                $crate::property::PropertyInfo(
                    Self::name().to_string(),
                    $crate::type_of::<Self>(),
                    Self::is_required(),
                    true
                )
            }

            fn compute(context: &$crate::context::Context, entity_id: $crate::EntityId) -> Option<Self> {
                // #[allow(unused_imports)]
                // use $crate::global_properties::ContextGlobalPropertiesExt;
                #[allow(unused_parens)]
                let ($($param,)*) = (
                    $(context.get_property_internal::<$dependency>(entity_id).unwrap()),*,
                    
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
        $derived_property:ident,
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
