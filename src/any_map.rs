/*!


Heterogeneous maps of the form `HashMap<Type, Vec<Type>>`. With this data structure you can do things like this:

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

use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use crate::type_of;


pub struct AnyMap {
  map: HashMap<TypeId, Box<dyn Any>>
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
    let v: &mut Vec<T> = v.downcast_mut().unwrap();
    v.push(value);
  }

  pub fn get_vec_mut<T: 'static>(& mut self) -> &mut Vec<T> {
    // self.map.get_mut(&type_of::<K>()).map(|v| v.downcast_mut::<Vec<K>>().unwrap())
    self.map
        .entry(type_of::<T>())
        .or_insert_with(| | { Box::new(Vec::<T>::new()) } )
        .downcast_mut()
        .unwrap()
  }

  pub fn get_vec_ref_unchecked<T: 'static>(& self) -> &Vec<T> {
    self.map.get(&type_of::<T>()).unwrap()
        .downcast_ref()
        .unwrap()
  }

  pub fn get_vec_ref<T: 'static>(& self) -> Option<&Vec<T>> {
    self.map.get(&type_of::<T>())
        .downcast_ref()
        .unwrap()
  }
}

mod tests {
  use super::AnyMap;

  #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
  struct Age(u8);
  #[derive(Clone, PartialEq, Eq, Hash, Debug)]
  struct Name(String);

  #[test]
  fn test_any_map() {

    let mut container = AnyMap::new();

    container.push(Age(49));
    container.push(Name("Robert".to_string()));

    {
      let vector = container.get_vec_mut::<Age>();
      assert_eq!(vector.len(), 1);
      assert_eq!(vector[0], Age(49));
    }

    {
      let vector: &mut Vec<Name> = container.get_vec_mut();
      assert_eq!(vector.len(), 1);
      assert_eq!(vector[0], Name("Robert".to_string()));
    }
  }

}
