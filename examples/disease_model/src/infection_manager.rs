use ixa_core::{
    Context, ContextPeopleExt, ContextRandomExt, EntityId, PersonPropertyChangeEvent, define_rng,
    trace,
};

use rand_distr::Exp;

use crate::INFECTION_DURATION;
use crate::people::{InfectionStatus, InfectionStatusValue};

pub type InfectionStatusEvent = PersonPropertyChangeEvent<InfectionStatus>;

define_rng!(InfectionRng);

fn schedule_recovery(context: &mut Context, entity_id: EntityId) {
    trace!("Scheduling recovery");
    let recovery_time = context.get_current_time()
        + context.sample_distr(InfectionRng, Exp::new(1.0 / INFECTION_DURATION).unwrap());
    context.add_plan(recovery_time, move |context| {
        context.set_person_property::<InfectionStatus>(
            entity_id,
            InfectionStatus,
            InfectionStatusValue::R,
        );
    });
}

fn handle_infection_status_change(context: &mut Context, event: InfectionStatusEvent) {
    trace!(
        "Handling infection status change from {:?} to {:?} for {:?}",
        event.previous, event.current, event.entity_id
    );
    if event.current == InfectionStatusValue::I {
        schedule_recovery(context, event.entity_id);
    }
}

pub fn init(context: &mut Context) {
    trace!("Initializing infection_manager");
    context.subscribe_to_event::<InfectionStatusEvent>(handle_infection_status_change);
}
