use seq_macro::seq;

use crate::{
    context::Context,
    entity::{
        ContextEntityExt,
        IndexValue,
        EntityData,
    },
    property::Property,
    EntityId,
    HashSet
};
use crate::entity::ContextEntityExtInternal;

/// Encapsulates a query.
///
/// [`Context::query_entities`] actually takes an instance of [`Query`], but because
/// we implement `Query` for tuples of up to size 20, that's invisible
/// to the caller. Do not use this trait directly.
pub trait Query {
    /// Registers each property in the query with the context and refreshes the indexes. Any work that requires
    /// a mutable reference to the context should be done here.
    fn setup(&self, context: &mut Context);
    /// Executes the query, accumulating the results with `accumulator`.
    fn execute_query(&self, context: &Context, accumulator: impl FnMut(EntityId));
    /// Checks that the given entity matches the query.
    fn match_entity(&self, context: &mut Context, entity: EntityId) -> bool;
}

// The empty query
impl Query for () {
    fn setup(&self, _: &mut Context) {}
    fn execute_query(&self, _context: &Context, _accumulator: impl FnMut(EntityId)){}
    fn match_entity(&self, _context: &mut Context, _entity: EntityId) -> bool { true }
}

// The query with one parameter
impl<T1: Property> Query for T1 {
    fn setup(&self, context: &mut Context) {
        if !context.is_registered::<T1>() {
            T1::register(context);
        }

        // 1. Refresh the indexes for each property in the query.
        let mut index_map = context.get_data_container::<EntityData>()
                                   .unwrap() // ToDo: Guarantee this unwrap doesn't panic.
                                   .property_indexes
                                   .borrow_mut();
        index_map.get_container_mut::<T1>().index_unindexed_entities(context);
    }

    fn execute_query(&self, context: &Context, mut accumulator: impl FnMut(EntityId)){
        // ToDo: Guarantee this unwrap doesn't panic.
        let entity_data = context.get_data_container::<EntityData>().unwrap();
        let index_map   = entity_data.property_indexes
                                     .borrow_mut();
        let mut indexes: Vec<&HashSet<EntityId>> = Vec::new();
        // A vector of closures that look up a property for an `entity_id`
        let mut unindexed: Vec<Box<dyn Fn(&EntityData, EntityId) -> bool>> = Vec::new();

        {
            // 1. Refresh the indexes for each property in the query.
            //    Done in setup.

            // 2. Collect the index entry corresponding to the value.
            let index = unsafe{ index_map.get_container_ref::<T1>().unwrap_unchecked() };
            let hash_value = IndexValue::new(&self);
            if let Some(lookup) = &index.lookup {
                if let Some(entities) = lookup.get(&hash_value) {
                    indexes.push(entities);
                } else {
                    // This is empty and so the intersection will also be empty.
                    return;
                }
            } else {
                // No index, so we'll get to this after.
                unindexed.push(
                    Box::new(move
                    |entity_data: &EntityData, entity_id: EntityId| {
                        match entity_data.get_property_ref::<T1>(entity_id) {
                            Some(value) => {
                                hash_value == IndexValue::new(value)
                            }
                            _ => { false }
                        }
                    })
                );
            }
        }

        // 3. Create an iterator over entities, based on either:
        //    (1) the smallest index if there is one.
        //    (2) the overall entity count if there are no indices.
        // let entity_data = context.get_data_container::<EntityData>().unwrap();
        let to_check: Box<dyn Iterator<Item =EntityId>> =
            if indexes.is_empty() {
                entity_data.entity_iterator()
            } else {
                let mut min_len: usize = usize::MAX;
                let mut shortest_idx: usize = 0;
                for (idx, index_iter) in indexes.iter().enumerate() {
                    if index_iter.len() < min_len {
                        shortest_idx = idx;
                        min_len = index_iter.len();
                    }
                }
                Box::new(indexes.remove(shortest_idx).iter().cloned())
            };

        // 4. Walk over the iterator and add entities to the result iff:
        //    (1) they exist in all the indexes
        //    (2) they match the unindexed properties
        'outer: for entity_id in to_check {
            // (1) check all the indexes
            for index in &indexes {
                if !index.contains(&entity_id) {
                    continue 'outer;
                }
            }

            // (2) check the unindexed properties
            for hash_lookup in &unindexed {
                if !hash_lookup(entity_data, entity_id) {
                    continue 'outer;
                }
            }

            // This matches.
            accumulator(entity_id);
        }
    }

    fn match_entity(&self, context: &mut Context, entity: EntityId) -> bool {
        match context.get_property::<T1>(entity) {

            Some(value) if &value == self => {
               true
            }

            _ => {
                // Either the value doesn't exist or it exists but doesn't match.
                false
            }

        }
    }
}

// Implement the versions with 1..20 parameters.
macro_rules! impl_query {
    ($ct:expr) => {
        seq!(N in 0..$ct {
            impl<
                #(
                    T~N : Property,
                )*
            > Query for (
                #(
                    T~N,
                )*
            )
            {
                fn setup(&self, context: &mut Context) {
                    #(
                        if !context.get_data_container_mut::<EntityData>()
                              .registered_derived_properties
                              .contains(&$crate::type_of::<T~N>())
                        {
                            <T~N>::register(context);
                        }
                    )*
                    // 1. Refresh the indexes for each property in the query.
                    let mut index_map = context.get_data_container::<EntityData>()
                                               .unwrap() // ToDo: Guarantee this unwrap doesn't panic.
                                               .property_indexes
                                               .borrow_mut();
                #(
                    index_map.get_container_mut::<T~N>().index_unindexed_entities(context);
                )*
                }

                fn execute_query(&self, context: &Context, mut accumulator: impl FnMut(EntityId)) {
                    // ToDo: Guarantee this unwrap doesn't panic.
                    let entity_data = context.get_data_container::<EntityData>().unwrap();
                    let index_map   = entity_data.property_indexes
                                                .borrow_mut();
                    let mut indexes: Vec<&HashSet<EntityId>> = Vec::new();
                    // A vector of closures that look up a property for an `entity_id`
                    let mut unindexed: Vec<Box<dyn Fn(&EntityData, EntityId) -> bool>> = Vec::new();

                    // 1. Refresh the indexes for each property in the query.
                    //    Done in setup.
                #(
                    {
                        // 2. Collect the index entry corresponding to the value.
                        // The following is guaranteed to be safe after the call to `get_container_mut` above.
                        let index = unsafe{ index_map.get_container_ref::<T~N>().unwrap_unchecked() };
                        let hash_value = IndexValue::new(&self.N);
                        if let Some(lookup) = &index.lookup {
                            if let Some(entities) = lookup.get(&hash_value) {
                                indexes.push(entities);
                            } else {
                                // This is empty and so the intersection will also be empty.
                                return;
                            }
                        } else {
                            // No index, so we'll get to this after.
                            unindexed.push(
                                Box::new(
                                    move
                                    |entity_data: &EntityData, entity_id: EntityId| {
                                        match entity_data.get_property_ref::<T~N>(entity_id) {
                                            Some(value) => {
                                                hash_value == IndexValue::new(value)
                                            }
                                            _ => { false }
                                        }
                                    }
                                )
                            );
                        }
                    }
                )*
                    // 3. Create an iterator over entities, based on either:
                    //    (1) the smallest index if there is one.
                    //    (2) the overall population if there are no indices.
                    let to_check: Box<dyn Iterator<Item = EntityId>> =
                        if indexes.is_empty() {
                            entity_data.entity_iterator()
                        } else {
                            let mut min_len: usize = usize::MAX;
                            let mut shortest_idx: usize = 0;
                            for (idx, index_iter) in indexes.iter().enumerate() {
                                if index_iter.len() < min_len {
                                    shortest_idx = idx;
                                    min_len = index_iter.len();
                                }
                            }
                            Box::new(indexes.remove(shortest_idx).iter().cloned())
                        };

                    // 4. Walk over the iterator and add entity to the result iff:
                    //    (1) they exist in all the indexes
                    //    (2) they match the unindexed properties
                    'outer: for entity_id in to_check {
                        // (1) check all the indexes
                        for index in &indexes {
                            if !index.contains(&entity_id) {
                                continue 'outer;
                            }
                        }

                        // (2) check the unindexed properties
                        for hash_lookup in &unindexed {
                            if !hash_lookup(entity_data, entity_id) {
                                continue 'outer;
                            }
                        }

                        // This matches.
                        accumulator(entity_id);
                    }
                }

                fn match_entity(&self, context: &mut Context, entity: EntityId) -> bool {
                    #(
                        match context.get_property::<T~N>(entity) {

                            Some(value) if value == self.N => {
                                /* pass through */
                            }

                            _ => {
                                // Either the value doesn't exist or it exists but doesn't match.
                                return false;
                            }

                        }
                    )*
                    // Matches every property in the query
                    true
                }
            }
        });
    }
}

seq!(Z in 1..20 {
    impl_query!(Z);
});

/// Helper utility for combining two queries, useful if you want
/// to iteratively construct a query in multiple parts.
///
/// Example:
/// ```ignore
/// use ixa_core::{Property, QueryAnd, Context, ContextPeopleExt};
///
/// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
/// struct Age(u8);
/// impl Property for Age {}
///
/// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
/// struct Alive(bool);
/// impl Property for Alive {}
///
/// let context = Context::new();
/// context.query_entities(QueryAnd::new(Age(42), Alive(true)));
/// ```
pub struct QueryAnd<Q1, Q2>
where
    Q1: Query,
    Q2: Query,
{
    queries: (Q1, Q2),
}

impl<Q1, Q2> QueryAnd<Q1, Q2>
where
    Q1: Query,
    Q2: Query,
{
    pub fn new(q1: Q1, q2: Q2) -> Self {
        Self { queries: (q1, q2) }
    }
}

// impl<Q1, Q2> Query for QueryAnd<Q1, Q2>
// where
//     Q1: Query,
//     Q2: Query,
// {
//     fn setup(&self, context: &mut Context) {
//         Q1::setup(&self.queries.0, context);
//         Q2::setup(&self.queries.1, context);
//     }
//
//     fn execute_query(&self, context: &Context, accumulator: impl FnMut(EntityId)) {
//         self.queries.0.execute_query(context, accumulator);
//     }
// }

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::define_derived_property;
    use crate::entity::data::EntityData;
    use crate::property::Property;
    use crate::entity::context_ext::{ContextEntityExt, ContextEntityExtInternal};

    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    struct Age(u8);
    impl Property for Age {}

    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    pub enum RiskCategory {
        High,
        Low,
    }
    impl Property for RiskCategory {}

    #[test]
    fn query_entities() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High).unwrap();

        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
    }


    #[test]
    fn query_entities_empty() {
        let mut context = Context::new();

        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn query_entities_count() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High).unwrap();

        assert_eq!(context.query_entity_count(RiskCategory::High), 1);
    }

    #[test]
    fn query_entity_count_empty() {
        let mut context = Context::new();

        assert_eq!(context.query_entity_count(RiskCategory::High), 0);
    }

    #[test]
    fn query_entity_macro_index_first() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&mut context));
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
    }

    fn property_is_indexed<T: Property>(context: &mut Context) -> bool {
        context
            .get_data_container_mut::<EntityData>()
            .get_index_ref::<T>()
            .unwrap()
            .lookup
            .is_some()
    }

    #[test]
    fn query_entity_macro_index_second() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High);
        let entities = context.query_entities(RiskCategory::High);
        assert!(!property_is_indexed::<RiskCategory>(&mut context));
        assert_eq!(entities.len(), 1);
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&mut context));
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn query_entity_macro_change() {
        let mut context = Context::new();
        let person1 = context.add_entity(RiskCategory::High).unwrap();

        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 0);

        context.set_property(person1, RiskCategory::High);
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 0);
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn query_entity_index_after_add() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&mut context));
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn query_entities_add_after_index() {
        let mut context = Context::new();
        let _ = context.add_entity(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&mut context));
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 1);

        let _ = context.add_entity(RiskCategory::High).unwrap();
        let entities = context.query_entities(RiskCategory::High);
        assert_eq!(entities.len(), 2);
    }

    #[test]
    // This is safe because we reindex only when someone queries.
    fn query_entities_add_after_index_without_query() {
        let mut context = Context::new();
        let _ = context.add_entity(()).unwrap();
        context.index_property::<RiskCategory>();
    }

    #[test]
    #[should_panic(expected = "Property not initialized")]
    // This will panic when we query.
    fn query_entities_add_after_index_panic() {
        let mut context = Context::new();
        context.add_entity(()).unwrap();
        context.index_property::<RiskCategory>();
        context.query_entities(RiskCategory::High);
    }

    #[test]
    fn query_entities_cast_value() {
        let mut context = Context::new();
        let _ = context.add_entity(Age(42)).unwrap();

        let entities = context.query_entities(Age(42));
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn query_entities_intersection() {
        let mut context = Context::new();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(40), RiskCategory::Low)).unwrap();

        let entities = context.query_entities((Age(42), RiskCategory::High));
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn query_entities_intersection_non_macro() {
        let mut context = Context::new();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(40), RiskCategory::Low)).unwrap();

        let entities = context.query_entities((Age(42), RiskCategory::High));
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn query_entities_intersection_one_indexed() {
        let mut context = Context::new();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_entity((Age(40), RiskCategory::Low)).unwrap();

        context.index_property::<Age>();
        let entities = context.query_entities((Age(42), RiskCategory::High));
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn query_derived_prop() {
        let mut context = Context::new();

        #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
        struct Senior(bool);
        define_derived_property!(Senior, [Age], |age| Some(Senior(age >= Age(65))));

        let person = context.add_entity(Age(64)).unwrap();
        let _ = context.add_entity(Age(88)).unwrap();

        // Age is a u8, by default integer literals are i32; the macro should cast it.
        let not_seniors = context.query_entities(Senior(false));
        let seniors = context.query_entities(Senior(true));
        assert_eq!(seniors.len(), 1, "One senior");
        assert_eq!(not_seniors.len(), 1, "One non-senior");

        context.set_property(person, Age(65));

        let not_seniors = context.query_entities(Senior(false));
        let seniors = context.query_entities(Senior(true));

        assert_eq!(seniors.len(), 2, "Two seniors");
        assert_eq!(not_seniors.len(), 0, "No non-seniors");
    }

    #[test]
    fn query_derived_prop_with_index() {
        let mut context = Context::new();

        #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
        struct Senior(bool);
        define_derived_property!(Senior, [Age], |age| Some(Senior(age >= Age(65))));

        context.index_property::<Senior>();
        let person = context.add_entity(Age(64)).unwrap();
        let _ = context.add_entity(Age(88)).unwrap();

        // Age is a u8, by default integer literals are i32; the macro should cast it.
        let not_seniors = context.query_entities(Senior(false));
        let seniors = context.query_entities(Senior(true));
        assert_eq!(seniors.len(), 1, "One senior");
        assert_eq!(not_seniors.len(), 1, "One non-senior");

        context.set_property(person, Age(65));

        let not_seniors = context.query_entities(Senior(false));
        let seniors = context.query_entities(Senior(true));

        assert_eq!(seniors.len(), 2, "Two seniors");
        assert_eq!(not_seniors.len(), 0, "No non-seniors");
    }
/*
    #[test]
    fn query_and_returns_entities() {
        let mut context = Context::new();
        context.add_entity((Age(42), RiskCategory::High)).unwrap();

        let entities = context.query_entities(QueryAnd::new(Age(42), RiskCategory::High));
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn query_and_conflicting() {
        let mut context = Context::new();
        context.add_entity((Age(42), RiskCategory::High)).unwrap();

        let entities = context.query_entities(QueryAnd::new(Age(42), Age(64)));
        assert_eq!(entities.len(), 0);
    }
*/
}
