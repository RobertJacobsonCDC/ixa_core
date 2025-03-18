// ToDo: Make this module generic over entity instead of specific to `PeopleId`

use crate::{
    context::Context,
    entity::ContextEntityExt,
    property::Property,
    type_of,
    EntityId,
    TypeId,
    HashMap, 
    HashSet
};
use std::{
    any::Any,
    hash::{Hash, Hasher},
    marker::PhantomData,
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
pub(crate) struct Index<T: Property> {
    // The hash of the property value maps to a list of EntityIds or None if we're not indexing.
    pub(super) lookup: Option<HashMap<IndexValue, HashSet<EntityId>>>,

    // The largest entity ID that has been indexed. Used so that we can lazily index when a
    // entity is added.
    pub(super) max_indexed: usize,

    phantom: PhantomData<T>,
}

impl<T: Property> Index<T> {
    pub(super) fn new() -> Self {
        Self {
            lookup: None,
            max_indexed: 0,
            phantom: PhantomData::default(),
        }
    }

    /// Looks up the value of the `T` property for `entity_id` and adds `entity_id` to the index
    /// set for that `value`.
    pub(crate) fn add_entity(&mut self, context: &Context, entity_id: EntityId) {
        let value = T::compute(context, entity_id);
        let value = value.unwrap_or_else(|| {
            // ToDo: This is what Ixa does, but it seems like we'd want to be able to query for people who do not have
            //       a value for a property. Have `None` hash to 0 or something.
            panic!(
                "{:?} has no {} value to index",
                entity_id,
                T::name()
            );
        });

        let index_value = IndexValue::new(&value);
        self.insert((entity_id, index_value));
    }

    /// Looks up the value of the `T` property for `entity_id` and removes `entity_id` from the
    /// index set for that `value`.
    fn remove_entity(&mut self, context: &mut Context, entity_id: EntityId) {
        let value = context.get_property::<T>(entity_id);
        // ToDo: If we index `None` values, we'd have to remove for None, too
        if let Some(value) = value {
            // ToDo: There is a lot of unwrapping here. What if values don't exist?
            let index_value = IndexValue::new(&value);
            let map: &mut HashMap<IndexValue, HashSet<EntityId>> = self.lookup.as_mut().unwrap();
            let set: &mut HashSet<EntityId> = map.get_mut(&index_value).unwrap();

            set.remove(&entity_id);
            // Clean up the entry if there are no people
            if set.is_empty() {
                map.remove(&index_value);
            }
        }
    }

    pub(crate) fn index_unindexed_entities(&mut self, context: &Context) {
        if self.lookup.is_none() {
            return;
        }
        let current_pop = context.get_entity_count();
        for id in self.max_indexed..current_pop {
            let entity_id = EntityId(id);
            self.add_entity(context, entity_id);
        }
        self.max_indexed = current_pop;
    }

    /// Inserts the `entity_id` into the index set for the given index value.
    pub(crate) fn insert(&mut self, (entity_id, index_value): (EntityId, IndexValue)) {
        // ToDo: Can `self.lookup` ever be `None` here?
        self.lookup
            .as_mut()
            .unwrap()
            .entry(index_value)
            .or_insert_with(HashSet::default)
            .insert(entity_id);
    }
}


// We don't use the `define_any_map_container!` macro, because the insert method inserts a
// `(EntityId, IndexValue)`, not a `T: Property`.
// define_any_map_container!(
//     IndexMap,
//     Index<T: Property>,
//     Index::<T>::new(),
//     Index::<T>::insert
// );

pub struct IndexMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for IndexMap{
    fn default() -> Self {
        Self::new()
    }
}

impl IndexMap {
    #[inline(always)]
    pub fn new() -> IndexMap {
        IndexMap {
            map: HashMap::default(),
        }
    }

    #[inline(always)]
    pub fn insert<T: Property>(&mut self, index: Index<T>) {
        let result = self.map.insert(type_of::<T>(), Box::new(index));
        // We shouldn't insert an index for a type that already has an index.
        assert!(result.is_none());
    }

    #[inline(always)]
    pub fn get_container_mut<T: Property + 'static>(&mut self) -> &mut Index<T> {
        unsafe {
            self.map
                .entry(type_of::<T>())
                .or_insert_with(|| Box::new(Index::<T>::new()))
                .downcast_mut()
                .unwrap_unchecked()
        }
    }

    #[inline(always)]
    pub fn get_container_ref<T: Property + 'static>(&self) -> Option<&Index<T>> {
        self.map
            .get(&type_of::<T>())
            .map(|v|
                unsafe {
                    v.downcast_ref()
                        .unwrap_unchecked()
                }
            )
    }

    #[inline(always)]
    pub unsafe fn get_container_ref_unchecked<T: Property + 'static>(&self) -> &Index<T> { unsafe {
        self.map
            .get(&type_of::<T>())
            .unwrap_unchecked()
            .downcast_ref()
            .unwrap_unchecked()
    }}

    #[inline(always)]
    pub fn contains_key(&self, type_of: &TypeId) -> bool {
        self.map.contains_key(type_of)
    }
}

/*
pub fn process_indices(
    context: &Context,
    remaining_indices: &[&Index],
    property_names: &mut Vec<String>,
    current_matches: &HashSet<EntityId>,
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

    use super::IndexValue;
    use crate::property::Property;

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    struct Age(u8);
    impl Property for Age {
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
