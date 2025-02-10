/*!

ToDo: Make this generic over entity.

*/

use std::collections::HashSet;
use seq_macro::seq;

use crate::{
    context::Context,
    people::{
        ContextPeopleExt,
        IndexValue,
        PeopleData,
    },
    property::Property,
    PersonId,
};

/// Encapsulates a person query.
///
/// [`Context::query_people`] actually takes an instance of [`Query`], but because
/// we implement Query for tuples of up to size 20, that's invisible
/// to the caller. Do not use this trait directly.
pub trait Query {
    /// Registers each property in the query with the context.
    fn setup(&self, context: &mut Context);
    /// Executes the query, accumulating the results with `accumulator`.
    fn execute_query(&self, context: &Context, accumulator: impl FnMut(PersonId));
}

// The empty query
impl Query for () {
    fn setup(&self, _: &mut Context) {}
    fn execute_query(&self, _context: &Context, _accumulator: impl FnMut(PersonId)){}
}

// The query with one parameter
impl<T1: Property> Query for T1 {
    fn setup(&self, context: &mut Context) {
        context.register_property::<T1>();
    }

    fn execute_query(&self, context: &Context, mut accumulator: impl FnMut(PersonId)){
        let people_data = context.get_data_container::<PeopleData>().unwrap();
        let mut index_map = people_data.property_indexes.borrow_mut();
        let mut indexes: Vec<&HashSet<PersonId>> = Vec::new();
        // A vector of closures that look up a property for a `person_id`
        let mut unindexed: Vec<Box<dyn Fn(&PeopleData, PersonId) -> bool>> = Vec::new();

        {
            // 1. Refresh the indexes for each property in the query.
            let index = index_map.get_container_mut::<T1>();
            index.index_unindexed_people(context);

            // 2. Collect the index entry corresponding to the value.
            let hash_value = IndexValue::new(&self);
            if let Some(lookup) = &index.lookup {
                if let Some(people) = lookup.get(&hash_value) {
                    indexes.push(people);
                } else {
                    // This is empty and so the intersection will also be empty.
                    return;
                }
            } else {
                // No index, so we'll get to this after.
                unindexed.push(
                    Box::new(move
                    |people_data: &PeopleData, person_id: PersonId| {
                        hash_value == IndexValue::new(
                            people_data.get_person_property_ref::<T1>(person_id)
                        )
                    })
                );
            }
        }

        // 3. Create an iterator over people, based on either:
        //    (1) the smallest index if there is one.
        //    (2) the overall population if there are no indices.
        let to_check: Box<dyn Iterator<Item = PersonId>> =
            if indexes.is_empty() {
                people_data.people_iterator()
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

        // 4. Walk over the iterator and add people to the result iff:
        //    (1) they exist in all the indexes
        //    (2) they match the unindexed properties
        'outer: for person_id in to_check {
            // (1) check all the indexes
            for index in &indexes {
                if !index.contains(&person_id) {
                    continue 'outer;
                }
            }

            // (2) check the unindexed properties
            for hash_lookup in &unindexed {
                if !hash_lookup(people_data, person_id) {
                    continue 'outer;
                }
            }

            // This matches.
            accumulator(person_id);
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
                        context.register_property::<T~N>();
                    )*
                }

                fn execute_query(&self, context: &Context, mut accumulator: impl FnMut(PersonId)) {
                    let people_data = context.get_data_container::<PeopleData>().unwrap();
                    let mut index_map = people_data.property_indexes.borrow_mut();
                    let mut indexes: Vec<&HashSet<PersonId>> = Vec::new();
                    // A vector of closures that look up a property for a `person_id`
                    let mut unindexed: Vec<Box<dyn Fn(&PeopleData, PersonId) -> bool>> = Vec::new();

                #(
                    {
                        // 1. Refresh the indexes for each property in the query.
                        let index = index_map.get_container_mut::<T~N>();
                        index.index_unindexed_people(context);
                    }
                )*
                #(
                    {
                        // 2. Collect the index entry corresponding to the value.
                        // The following is guaranteed to be safe after the call to `get_container_mut` above.
                        let index = unsafe{ index_map.get_container_ref::<T~N>().unwrap_unchecked() };
                        let hash_value = IndexValue::new(&self.N);
                        if let Some(lookup) = &index.lookup {
                            if let Some(people) = lookup.get(&hash_value) {
                                indexes.push(people);
                            } else {
                                // This is empty and so the intersection will also be empty.
                                return;
                            }
                        } else {
                            // No index, so we'll get to this after.
                            unindexed.push(
                                Box::new(
                                    move
                                    |people_data: &PeopleData, person_id: PersonId| {
                                        hash_value == IndexValue::new(
                                            people_data.get_person_property_ref::<T~N>(person_id)
                                        )
                                    }
                                )
                            );
                        }
                    }
                )*
                    // 3. Create an iterator over people, based on either:
                    //    (1) the smallest index if there is one.
                    //    (2) the overall population if there are no indices.
                    let to_check: Box<dyn Iterator<Item = PersonId>> =
                        if indexes.is_empty() {
                            people_data.people_iterator()
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

                    // 4. Walk over the iterator and add people to the result iff:
                    //    (1) they exist in all the indexes
                    //    (2) they match the unindexed properties
                    'outer: for person_id in to_check {
                        // (1) check all the indexes
                        for index in &indexes {
                            if !index.contains(&person_id) {
                                continue 'outer;
                            }
                        }

                        // (2) check the unindexed properties
                        for hash_lookup in &unindexed {
                            if !hash_lookup(people_data, person_id) {
                                continue 'outer;
                            }
                        }

                        // This matches.
                        accumulator(person_id);
                    }
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
/// use ixa_properties::{Property, QueryAnd, Context, ContextPeopleExt};
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
/// context.query_people(QueryAnd::new(Age(42), Alive(true)));
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
//     fn execute_query(&self, context: &Context, accumulator: impl FnMut(PersonId)) {
//         self.queries.0.execute_query(context, accumulator);
//     }
// }

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::define_derived_property;
    use crate::people::people_data::PeopleData;
    use crate::property::Property;
    use std::any::TypeId;
    use crate::people::context_ext::ContextPeopleExt;

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
    fn query_people() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High).unwrap();

        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_empty() {
        let context = Context::new();

        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 0);
    }

    #[test]
    fn query_people_count() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High).unwrap();

        assert_eq!(context.query_people_count(RiskCategory::High), 1);
    }

    #[test]
    fn query_people_count_empty() {
        let context = Context::new();

        assert_eq!(context.query_people_count(RiskCategory::High), 0);
    }

    #[test]
    fn query_people_macro_index_first() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&context));
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
    }

    fn property_is_indexed<T: 'static>(context: &Context) -> bool {
        context
            .get_data_container::<PeopleData>()
            .unwrap()
            .get_index_ref(TypeId::of::<T>())
            .unwrap()
            .lookup
            .is_some()
    }

    #[test]
    fn query_people_macro_index_second() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High);
        let people = context.query_people(RiskCategory::High);
        assert!(!property_is_indexed::<RiskCategory>(&context));
        assert_eq!(people.len(), 1);
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&context));
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_macro_change() {
        let mut context = Context::new();
        let person1 = context.add_person(RiskCategory::High).unwrap();

        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 0);

        context.set_person_property(person1, RiskCategory::High);
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 0);
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_index_after_add() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&context));
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_add_after_index() {
        let mut context = Context::new();
        let _ = context.add_person(RiskCategory::High).unwrap();
        context.index_property::<RiskCategory>();
        assert!(property_is_indexed::<RiskCategory>(&context));
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 1);

        let _ = context.add_person(RiskCategory::High).unwrap();
        let people = context.query_people(RiskCategory::High);
        assert_eq!(people.len(), 2);
    }

    #[test]
    // This is safe because we reindex only when someone queries.
    fn query_people_add_after_index_without_query() {
        let mut context = Context::new();
        let _ = context.add_person(()).unwrap();
        context.index_property::<RiskCategory>();
    }

    #[test]
    #[should_panic(expected = "Property not initialized")]
    // This will panic when we query.
    fn query_people_add_after_index_panic() {
        let mut context = Context::new();
        context.add_person(()).unwrap();
        context.index_property::<RiskCategory>();
        context.query_people(RiskCategory::High);
    }

    #[test]
    fn query_people_cast_value() {
        let mut context = Context::new();
        let _ = context.add_person(Age(42)).unwrap();

        // Age is a u8, by default integer literals are i32; the macro should cast it.
        let people = context.query_people(Age(42));
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_intersection() {
        let mut context = Context::new();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person(((Age(40), RiskCategory::Low))).unwrap();

        let people = context.query_people((Age(42), RiskCategory::High));
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_intersection_non_macro() {
        let mut context = Context::new();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person(((Age(40), RiskCategory::Low))).unwrap();

        let people = context.query_people((Age(42), RiskCategory::High));
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_people_intersection_one_indexed() {
        let mut context = Context::new();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person((Age(42), RiskCategory::High)).unwrap();
        let _ = context.add_person(((Age(40), RiskCategory::Low))).unwrap();

        context.index_property(Age);
        let people = context.query_people((Age(42), RiskCategory::High));
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_derived_prop() {
        let mut context = Context::new();
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        struct Senior(bool);
        define_derived_property!(Senior, [Age], |age| Senior(age >= Age(65)));

        let person = context.add_person(Age(64)).unwrap();
        let _ = context.add_person(Age(88)).unwrap();

        // Age is a u8, by default integer literals are i32; the macro should cast it.
        let not_seniors = context.query_people(Senior(false));
        let seniors = context.query_people(Senior(true));
        assert_eq!(seniors.len(), 1, "One senior");
        assert_eq!(not_seniors.len(), 1, "One non-senior");

        context.set_person_property(person, Age(65));

        let not_seniors = context.query_people(Senior(false));
        let seniors = context.query_people(Senior(true));

        assert_eq!(seniors.len(), 2, "Two seniors");
        assert_eq!(not_seniors.len(), 0, "No non-seniors");
    }

    #[test]
    fn query_derived_prop_with_index() {
        let mut context = Context::new();
        define_derived_property!(Senior, bool, [Age], |age| age >= 65);

        context.index_property(Senior);
        let person = context.add_person((Age, 64)).unwrap();
        let _ = context.add_person((Age, 88)).unwrap();

        // Age is a u8, by default integer literals are i32; the macro should cast it.
        let not_seniors = context.query_people(Senior(false));
        let seniors = context.query_people(Senior(true));
        assert_eq!(seniors.len(), 1, "One senior");
        assert_eq!(not_seniors.len(), 1, "One non-senior");

        context.set_person_property(person, Age, 65);

        let not_seniors = context.query_people(Senior(false));
        let seniors = context.query_people(Senior(true));

        assert_eq!(seniors.len(), 2, "Two seniors");
        assert_eq!(not_seniors.len(), 0, "No non-seniors");
    }

    #[test]
    fn query_and_returns_people() {
        let mut context = Context::new();
        context.add_person((Age(42), RiskCategory::High)).unwrap();

        let people = context.query_people(QueryAnd::new(Age(42), RiskCategory::High));
        assert_eq!(people.len(), 1);
    }

    #[test]
    fn query_and_conflicting() {
        let mut context = Context::new();
        context.add_person((Age(42), RiskCategory::High)).unwrap();

        let people = context.query_people(QueryAnd::new(Age(42), Age(64)));
        assert_eq!(people.len(), 0);
    }
}
*/
