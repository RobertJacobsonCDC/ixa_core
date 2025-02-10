use crate::new_trait::New;
use crate::type_of;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Context {
    // This is actually a `HashMap<TypeId, Box<dyn New>>` but must be declared this way to avoid 
    // having to implement an `as_any()` method on everything, at least as far as I know.
    data_plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            data_plugins: HashMap::new(),
        }
    }

    /// Returns a mutable reference for the data container for `T`, creating it if it doesn't exist yet.
    pub fn get_data_container_mut<T: New>(&mut self) -> &mut T {
        self.data_plugins
            .entry(type_of::<T>())
            .or_insert_with(|| Box::new(<T as New>::new()))
            .downcast_mut::<T>()
            .unwrap() // Will never panic as data container has the matching type
    }

    /// Returns a reference to the data container for `T` if it exists.
    /// If you need a mutable reference or lazy instantiation, use `Context::get_data_container_mut()`.
    pub fn get_data_container<T: New>(&self) -> Option<&T> {
        if let Some(data) = self.data_plugins.get(&type_of::<T>()) {
            data.downcast_ref::<T>()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let mut context = Context::new();
        {
            // If you specify the type of the variable the compiler can infer the generic type.
            let byte_vector: &mut Vec<u8> = context.get_data_container_mut();
            byte_vector.push(1);
            byte_vector.push(2);
            byte_vector.push(3);
        }
        {
            let byte_vector: &mut Vec<&str> = context.get_data_container_mut();
            byte_vector.push("4");
            byte_vector.push("5");
            byte_vector.push("6");
        }

        let result = context.get_data_container::<Vec<u8>>();
        assert!(result.is_some());
        println!("{:?}", result.unwrap());

        let result = context.get_data_container::<Vec<&str>>();
        assert!(result.is_some());
        println!("{:?}", result.unwrap());
    }
}
