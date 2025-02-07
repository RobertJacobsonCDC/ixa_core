/*!

Concrete `Vec` `AnyMap` and a macro to implement the `AnyMap` pattern for any static type.

All `unsafe` in the implementation can be removed if you'd like, but each use is safe by 
construction.

`AnyMap` is a heterogeneous map of the form `HashMap<Type, Vec<Type>>`. With this data structure 
you can do things like this:

```rust
use crate::AnyMap;

# #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Age(u8);
# #[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Name(String);
# #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Height(u32);
# #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum InfectionStatus {
  Susceptible,
  Infected,
  Recovered
}

# fn main() {
let mut container = AnyMap::new();

container.push(Age(37u8));
container.push(Name(format!("Name {}", "Robert")));
container.push(Height(121u32));
container.push(InfectionStatus::Recovered);

let height: Height = container.pop().unwrap();
# }
```

 - `AnyMap` is implemented using `std::any::Any`
 - `Type` must have `'static` lifetime or be a value type
 - There's nothing special about `Vec<T>`'s. You can implement the same thing for any container, including custom containers.


You can do this in ~2X the speed and without the `'static` restriction if you're willing to go unsafe

*/

use crate::{
    type_of,
    TypeId
};
use std::{
    any::Any,
    collections::HashMap
};

pub struct AnyMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl AnyMap {
    pub fn new() -> AnyMap {
        AnyMap {
            map: HashMap::new(),
        }
    }

    pub fn push<T: 'static>(&mut self, value: T) {
        let v = self
            .map
            .entry(type_of::<T>())
            .or_insert_with(|| Box::new(Vec::<T>::new()));
        // ToDo: Use `Any::downcast_mut_unchecked` (nightly feature). This is guaranteed safe, 
        //       because only a `Box<Vec<T>>` can be mapped to by `type_of::<T>()`.
        let v: &mut Vec<T> = unsafe { v.downcast_mut().unwrap_unchecked() };
        v.push(value);
    }

    pub fn get_container_mut<T: 'static>(&mut self) -> &mut Vec<T> {
        unsafe {
            self.map
                .entry(type_of::<T>())
                .or_insert_with(|| Box::new(Vec::<T>::new()))
                // ToDo: Use `Any::downcast_mut_unchecked` (nightly feature). This is guaranteed safe, 
                //       because only a `Box<Vec<T>>` can be mapped to by `type_of::<T>()`.
                .downcast_mut()
                .unwrap_unchecked()
        }
    }

    pub unsafe fn get_container_ref_unchecked<T: 'static>(&self) -> &Vec<T> {
        self.map
            .get(&type_of::<T>())
            .unwrap_unchecked() // This is unsafe; the caller must guarantee the type exists.
            // ToDo: Use `Any::downcast_mut_unchecked` (nightly feature). This is guaranteed safe, 
            //       because only a `Box<Vec<T>>` can be mapped to by `type_of::<T>()`.
            .downcast_ref()
            .unwrap_unchecked()
    }

    pub fn get_container_ref<T: 'static>(&self) -> Option<&Vec<T>> {
        // ToDo: Use `Any::downcast_ref_unchecked` (nightly feature). This is guaranteed safe, 
        //       because only a `Vec<T>` can be mapped to by `type_of::<T>()`.
        self.map.get(&type_of::<T>()).map(|v| unsafe { v.downcast_ref().unwrap_unchecked() })
    }
}

/// Defines a container struct implementing the `AnyMap` pattern, a container that can store 
/// multiple types of values, routing values to the right variant of inner container type. The 
/// macro generates a struct with methods to insert values and to retrieve the inner container for 
/// a given type.
///
/// # Parameters
/// - `$name`: The name of the struct to define.
/// - `$container<$generic $( : $traitfirst $(+ $traitrest)* )?>`: The type of container to use for 
///    each generic type with optional trait constraints (e.g., `Vec<T: Property + Send>`). The 
///    additional constraint that `$generic` is `'static` will be added automatically.
/// - `$constructor`: An expression to construct a new instance of the container.
/// - `$inserter`: A function or closure to insert a value into the container.
///
/// # Example
/// ```
/// # use ixa_properties::any_map::define_any_map_container;
/// define_any_map_container!(
///     MyContainer,
///     Vec<T: Clone>,
///     Vec::<T>::new(),
///     Vec::push
/// );
///
/// let mut container = MyContainer::new();
/// container.push(42); // Inserts 42 into a `Vec<i32>`
/// container.push("hello"); // Inserts "hello" into a `Vec<&str>`
/// ```
///
/// # Safety
/// - The `get_container_ref_unchecked` method is unsafe because it uses `unwrap_unchecked`
///   to bypass runtime checks. Ensure the type exists in the map before calling this method.

#[macro_export]
macro_rules! define_any_map_container {
    (
        $name:ident,
        $container:ident<$generic:ident $( : $traitfirst:ident $(+ $traitrest:ident)* )?>,
        $constructor:expr,
        $inserter:expr
    ) => {
        pub struct $name {
            map: std::collections::HashMap<$crate::TypeId, Box<dyn std::any::Any>>,
        }

        impl $name {
            #[inline]
            pub fn new() -> $name {
                $name {
                    map: std::collections::HashMap::new(),
                }
            }

            #[inline]
            pub fn push<$generic : $( $traitfirst $(+ $traitrest)* +)? 'static>(&mut self, value: $generic) {
                let v: &mut $container<$generic> = self.get_container_mut();
                ($inserter)(v, value);
            }

            #[inline]
            pub fn get_container_mut<$generic : $( $traitfirst $(+ $traitrest)* +)? 'static>(&mut self) -> &mut $container<$generic> {
                unsafe {
                    self.map
                        .entry($crate::type_of::<$generic>())
                        .or_insert_with(|| Box::new($constructor))
                        .downcast_mut()
                        .unwrap_unchecked() // This is always safe
                }
            }
            
            #[inline]
            pub fn get_container_ref<$generic : $( $traitfirst $(+ $traitrest)* +)? 'static>(&self) -> Option< & $container< $generic > > {
                self.map
                    .get(&$crate::type_of::<$generic>())
                    .map(|v| 
                        unsafe { 
                            v.downcast_ref()
                             .unwrap_unchecked() // This is always safe
                        }
                    )
            }
            
            #[inline]
            pub unsafe fn get_container_ref_unchecked<$generic : $( $traitfirst $(+ $traitrest)* +)? 'static>(&self) -> &$container<$generic> {
                self.map
                    .get(&$crate::type_of::<$generic>())
                    .unwrap_unchecked() // The caller must guarantee this is safe
                    .downcast_ref()
                    .unwrap_unchecked() // This is always safe
            }
        }
    };
}
pub use define_any_map_container;



mod tests {
    use std::hash::Hash;
    use super::define_any_map_container;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct Age(u8);
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct Name(String);

    define_any_map_container!(
        MyContainer,
        Vec<T: Clone + Hash>,
        Vec::<T>::new(),
        Vec::push
    );

    #[test]
    fn test_any_map() {
        let mut container = MyContainer::new();

        container.push(Age(49));
        container.push(Name("Robert".to_string()));

        {
            let vector = container.get_container_mut::<Age>();
            assert_eq!(vector.len(), 1);
            assert_eq!(vector[0], Age(49));
        }

        {
            let vector: &mut Vec<Name> = container.get_container_mut();
            assert_eq!(vector.len(), 1);
            assert_eq!(vector[0], Name("Robert".to_string()));
        }
    }
}
