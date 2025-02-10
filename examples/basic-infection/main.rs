mod incidence_report;
mod infection_manager;
mod people;
mod transmission_manager;

use ixa_properties::Context;
use ixa_properties::IxaError;
use ixa_properties::random::ContextRandomExt;

static POPULATION: u64 = 1000;
static SEED: u64 = 123;
static MAX_TIME: f64 = 303.0;
static FOI: f64 = 0.1;
static INFECTION_DURATION: f64 = 5.0;

fn initialize(context: &mut Context) -> Result<(), IxaError> {
    context.init_random(SEED);

    people::init(context);
    transmission_manager::init(context);
    infection_manager::init(context);
    incidence_report::init(context)?;

    context.add_plan(MAX_TIME, |context| {
        context.shutdown();
    });
    Ok(())
}

fn main() {
    let mut context = ixa_properties::Context::default();

    initialize(&mut context).expect("Failed to initialize Context");
    context.execute();
}
