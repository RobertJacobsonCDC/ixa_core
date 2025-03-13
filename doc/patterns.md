---
marp: true
theme: hegel
footer: Robert Jacobson, 10 February 2025
style: footer {text-align: right;} h1,h2,h3 {color: #ED706B;}
# style: footer {text-align: right;} h1,h2,h3 {color: #09c;}
class: invert
---

<!--
_footer: ""
-->

# Rust Patterns

![bg right](designpatterns_inverted.png)

> Robert Jacobson, 10 February 2025

---

# Trait Objects

A trait is object safe if it can be made into a trait object: `Box<dyn MyTrait>` / `&dyn Trait`. Requirements:

 - No static methods. All methods take `&self`, `&mut self`, etc.
 - No `Self` type parameter or return types.
 - No generics. `fn foo<T: Property>(&self) -> T` not allowed.
 - No `impl trait` return types.

---

# `std::any::Any` Trait for `'static` Types

```rust
use std::any::Any;

trait Foo: Any {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
struct Age(u8);
impl Foo for Age {
    fn as_any(&self) -> &dyn Any { self }
}
#[derive(Debug)]
struct Name(&'static str);
impl Foo for Name {
    fn as_any(&self) -> &dyn Any { self }
}
```

---

# Upcast to Downcast

```rust
fn main() {
    let mut v = Vec::<Box<dyn Foo>>::new();
    v.push(Box::new(Age(43)));
    v.push(Box::new(Name("Robert")));

    println!("Popped: {:?}", v.pop().unwrap().as_any().downcast_ref::<Name>().unwrap());
    println!("Popped: {:?}", v.pop().unwrap().as_any().downcast_ref::<Age>().unwrap());
}
```

```
Popped: Name("Robert")
Popped: Age(43)
```

```rust
      &dyn Any
      ╱     ╲
&dyn Foo     ╲
             &Age
```

---
