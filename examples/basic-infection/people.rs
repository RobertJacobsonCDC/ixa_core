use ixa_properties::{Context, ContextPeopleExt, Property};
use log::trace;

// use serde::{Deserialize, Serialize};

use crate::POPULATION;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum InfectionStatus {
    S,
    I,
    R,
}
impl Property for InfectionStatus {}


/// Populates the "world" with the `POPULATION` number of people.
pub fn init(context: &mut Context) {
    trace!("Initializing people");
    for _ in 0..POPULATION {
        context.add_person(()).unwrap();
    }
}
