use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    marker::PhantomData
};
use crate::{
    PersonId,
    type_of,
    context::Context,
    people::ContextPeopleExt,
    property::Property
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
// The lookup key for entries in the index. This is a serialized
// version of the value. If that serialization fits in 128 bits, we
// store it in Fixed to avoid the allocation of the Vec. Otherwise, it
// goes in Variable.
#[doc(hidden)]
pub enum IndexValue {
    Fixed(u128),
    Variable(Vec<u8>),
}

impl IndexValue {
    pub fn new<T: Hash>(val: &T) -> IndexValue {
        let mut hasher = IndexValueHasher::new();
        val.hash(&mut hasher);
        if hasher.buf.len() <= 16 {
            let mut tmp: [u8; 16] = [0; 16];
            tmp[..hasher.buf.len()].copy_from_slice(&hasher.buf[..]);
            return IndexValue::Fixed(u128::from_le_bytes(tmp));
        }
        IndexValue::Variable(hasher.buf)
    }
}

// Implementation of the Hasher interface for IndexValue, used
// for serialization. We're actually abusing this interface
// because you can't call finish().
struct IndexValueHasher {
    buf: Vec<u8>,
}

impl IndexValueHasher {
    fn new() -> Self {
        IndexValueHasher { buf: Vec::new() }
    }
}

impl Hasher for IndexValueHasher {
    fn finish(&self) -> u64 {
        panic!("Unimplemented")
    }

    fn write(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }
}

// An index for a single property.
pub struct Index<T: Property> {
    // The hash of the property value maps to a list of PersonIds
    // or None if we're not indexing
    pub(super) lookup: Option<HashMap<IndexValue, HashSet<PersonId>>>,

    // The largest person ID that has been indexed. Used so that we
    // can lazily index when a person is added.
    pub(super) max_indexed: usize,

    phantom: PhantomData<T>,
}

impl<T: Property> Index<T> {
    pub(super) fn new() -> Self {
        Self {
            lookup: None,
            max_indexed: 0,
            phantom: PhantomData::default()
        }
    }

    pub(super) fn add_person(&mut self, context: &mut Context, person_id: PersonId) {
        let value = context.get_person_property::<T>(person_id);
        let value = value.unwrap_or_else(|_| {
            // ToDo: This is what Ixa does, but it seems like we'd want to be able to query for people who do not have
            //       a value for a property. Have `None` hash to 0 or something.
            panic!("{:?} not found has no {} value to index", person_id, T::name());
        });

        let hash = IndexValue::new(&value);
        self.lookup
            .as_mut()
            .unwrap()
            .entry(hash)
            .or_insert_with(HashSet::new)
            .insert(person_id);
    }

    pub(super) fn remove_person(
        &mut self,
        context: &mut Context,
        person_id: PersonId,
    ) {
        let value = context.get_person_property::<T>(person_id);
        // ToDo: If we index `None` values, we'd have to remove for None, too
        if let Some(value) = value {
            // ToDo: There is a lot of unwrapping here. What if values don't exist?
            let index_value = IndexValue::new(&value);
            let map: &mut HashMap<IndexValue, HashSet<PersonId>> = self.lookup.as_mut().unwrap();
            let set: &mut HashSet<PersonId> = map.get_mut(&index_value).unwrap();
            
            set.remove(&person_id);
            // Clean up the entry if there are no people
            if set.is_empty() {
                map.remove(&index_value);
            }
        }
    }

    pub(super) fn index_unindexed_people(&mut self, context: &mut Context) {
        if self.lookup.is_none() {
            return;
        }
        let current_pop = context.get_current_population();
        for id in self.max_indexed..current_pop {
            let person_id = PersonId(id);
            self.add_person(context, person_id);
        }
        self.max_indexed = current_pop;
    }
}


/// A map from `TypeId` to `Index<T>`. This follows the `AnyMap` pattern. This is what `PeopleData` uses to
/// look up `Index`es.
pub(crate) struct IndexMap {
    /// This is actually a HashMap<TypeId, Box<Index<T>>>
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl IndexMap {
    pub fn insert<T: Property> (&mut self, index: Index<T>) {
        self.map.insert(type_of::<T>(), Box::new(index)).expect("failed to insert type Index into IndexMap");
    }

    #[must_use]
    pub fn get<T: Property>(&self) -> Option<&Index<T>> {
        let index = self.map.get(&type_of::<T>());
        index.downcast_ref()
    }

    #[must_use]
    pub fn get_mut<T: Property>(&self) -> Option<&mut Index<T>> {
        let index = self.map.get(&type_of::<T>());
        index.downcast_mut()
    }
}

/*
pub fn process_indices(
    context: &Context,
    remaining_indices: &[&Index],
    property_names: &mut Vec<String>,
    current_matches: &HashSet<PersonId>,
    print_fn: &dyn Fn(&Context, &[String], usize),
) {
    if remaining_indices.is_empty() {
        print_fn(context, property_names, current_matches.len());
        return;
    }

    let (next_index, rest_indices) = remaining_indices.split_first().unwrap();
    let lookup = next_index.lookup.as_ref().unwrap();

    // If there is nothing in the index, we don't need to process it
    if lookup.is_empty() {
        return;
    }

    for (display, people) in lookup.values() {
        let intersect = !property_names.is_empty();
        property_names.push(display.clone());

        let matches = if intersect {
            &current_matches.intersection(people).copied().collect()
        } else {
            people
        };

        process_indices(context, rest_indices, property_names, matches, print_fn);
        property_names.pop();
    }
}
*/

#[cfg(test)]
mod test {
    // Tests in `src/people/query.rs` also exercise indexing code.

    use crate::context::Context;
    use crate::property::Property;
    use super::{Index, IndexValue};

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    struct Age(u8);
    impl Property for Age{
        fn name() -> &'static str {
            "Age"
        }
    }

    #[test]
    fn test_index_value_hasher_finish2_short() {
        let value = 42;
        let index = IndexValue::new(&value);
        assert!(matches!(index, IndexValue::Fixed(_)));
    }

    #[test]
    fn test_index_value_hasher_finish2_long() {
        let value = "this is a longer string that exceeds 16 bytes";
        let index = IndexValue::new(&value);
        assert!(matches!(index, IndexValue::Variable(_)));
    }

    #[test]
    fn test_index_value_compute_same_values() {
        let value = "test value";
        let value2 = "test value";
        assert_eq!(IndexValue::new(&value), IndexValue::new(&value2));
    }

    #[test]
    fn test_index_value_compute_different_values() {
        let value1 = 42;
        let value2 = 43;
        assert_ne!(IndexValue::new(&value1), IndexValue::new(&value2));
    }
}
