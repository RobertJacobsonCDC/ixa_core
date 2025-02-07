/*!

An object safe trait for types that know how to construct themselves with `new()`. The hitch 
is it needs to be `'static` or a value type. (Note that such a type automatically implements 
`std::any::Any`.) Fortunately, a lot of types satisfy those requirements:

 - `Vec<T>` for `T: 'static`
 - primitive types like `u8`, `f64`, `usize`, ...
 - `Vec<i32>` and other `std` containers of primitive types
 - `String`
 - `&'static str`

The syntax is identical to calling a `new()` constructor on any other type:

```rust
# use ixa_properties::New;

struct MyStruct {
  a: u32,
  b: &'static str
}

impl New for MyStruct{
  const new: &'static dyn Fn() -> Self = &||MyStruct{ a: 0, b: ""}; 
}

# fn do_something(){
let my_struct = MyStruct::new();
// ...
# }
```

If your type already has a `new()` method, implementing `New` is a cinch, but you need
to do the standard ceremony for calling `New::new()` when there's a name collision:

```rust
# use ixa_properties::New;
struct MyStruct {
  a: u32,
  b: &'static str
}

impl MyStruct{
  pub fn new() -> Self {
    MyStruct{ a: 0, b: ""}
  }
}

impl New for MyStruct{
  const new: &'static dyn Fn() -> Self = &MyStruct::new; 
}

# fn foo(){
let my_struct = <MyStruct as New>::new();
// ...
# } 
```

*/

use std::any::Any;

/// An object-safe trail that can construct itself.
pub trait New: Any + 'static {
  /// A constant reference to a constructor
  #[allow(non_upper_case_globals)]
  const new: &'static dyn Fn() -> Self;
}

// This is how you would implement this for your types.
impl<T: 'static> New for Vec<T> {
  const new: &'static dyn Fn() -> Self = &Vec::<T>::new;
}

impl New for String {
  const new: &'static dyn Fn() -> Self = &String::new;
}

#[cfg(test)]
mod tests {
  use std::any::Any;
  use std::collections::HashMap;
  use super::*;

  #[test]
  fn test_new_vec() {
    let mut str_vec = Vec::<&str>::new();
    str_vec.push(&"duck");
  }
  
  #[test]
  fn test_as_any() {
    let mut map: HashMap<&str, Box<dyn Any>> = HashMap::new();
    let any_vec: Box<dyn Any> = Box::new( <Vec::<u8> as New>::new() );
    map.insert("duck", any_vec);
  }
}
