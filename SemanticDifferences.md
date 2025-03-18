Surely `ContextPeopleExtInteral` should be `pub(crate)` or private:

```rust
pub trait ContextPeopleExtInternal...
```

# Best Practice and Style

 - `get_ref() -> &T` and `get_mut() -> &mut T`
 - `v.unwrap_or_else(| | panic!("message"))` vs `v.unwrap().expect("message")`
 - `pub fn mutating_method(&mut self) { ... }` vs `pub fn mutating_method(&self) { ... }` 
 - 

# "Semantic" Differences

 - Global properties are global to the `Context` instance, not global to the program.
 - Since it's only called for derived properties, `Property::dependencies` is now `Property::collect_dependencies(dependencies: &mut Vec<TypeId>)` and pushes `type_of<Self>()` if the property is not derived.
 - Some `panic`s in cases of lazily created objects not existing have been turned into just the construction of the object. 
   - A property can be queried before it's assigned



# Questions / Comments

 - For `T: Property` we have `T` $\to$ `String`, and `T` $\to$ `TypeId`. 
   - Tabulator and probably debugger will want `String` $\to$ `TypeId`.
   - Does anything want  `TypeId`$\to$ `String`?
   - Anything $\to$ `T` is harder. We could map to an `impl Property` instance and call static methods if we really need to.
 - A call `thing.get::<T: Property>()` eventually "bottoms out" at some lookup `person_data.some_map.get(type_id)` where `type_id = type_of::<T>()`. At what point do we change from `func_with_generic::<T>()` to `generic_func(type_id: TypeId)`? Answer: The public API gets `func_with_generic::<T>()`, and for internal implementation the chain stops when we no longer need `T`, only `type_id: TypdId`.  So internal implementation gets `func_with_generic::<T>()` when the implementation is genuinely different for different `T` (e.g. calls a static `T::something()`),  `generic_func(type_id: TypeId)` otherwise.
