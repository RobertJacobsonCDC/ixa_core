/*!

Much of this could be factored out to be generic with respect to entity rather than specific to 
people. The `property`, `property_map`, and `init_list` modules already are.

*/

mod context_ext;
mod people_data;
mod index;
mod query;
mod init_list;

// `ContextPeopleExt` is the public API to `PeopleData`.
pub(crate) use people_data::PeopleData;
pub(crate) use init_list::InitializationList;
pub(crate) use context_ext::ContextPeopleExtInternal;
pub(crate) use index::{Index, IndexMap, IndexValue};
pub(crate) use query::Query;

pub use context_ext::ContextPeopleExt;
