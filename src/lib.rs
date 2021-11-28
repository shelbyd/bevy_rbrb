use bevy_app::*;
use bevy_ecs::{prelude::*, system::ExclusiveSystem};
use std::time::Duration;

pub use rbrb::*;

mod snapshot;
mod stage;
use stage::*;

pub struct RbrbPlugin;

impl Plugin for RbrbPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_before(CoreStage::Update, "rbrb_update", RbrbStage::new());
    }
}

pub struct RbrbTime {
    pub delta: Duration,
}

pub trait RbrbAppExt {
    fn with_session(&mut self, session: rbrb::Session) -> &mut Self;
    fn with_typed_input_system<
        I: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        Params,
        S: System<In = PlayerId, Out = I>,
    >(
        &mut self,
        system: impl IntoSystem<Params, S>,
    ) -> &mut Self;

    fn update_rollback_schedule(&mut self, f: impl FnOnce(&mut Schedule)) -> &mut Self;
}

impl RbrbAppExt for AppBuilder {
    fn with_session(&mut self, session: rbrb::Session) -> &mut Self {
        self.insert_resource(session);
        self
    }

    fn with_typed_input_system<
        I: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        Params,
        S: System<In = PlayerId, Out = I>,
    >(
        &mut self,
        system: impl IntoSystem<Params, S>,
    ) -> &mut Self {
        let mut get_inputs = Box::new(system.system().chain(serialize_inputs.system()));
        get_inputs.initialize(self.world_mut());

        let mut parse_inputs = Box::new(parse_inputs::<I>.exclusive_system());
        parse_inputs.initialize(self.world_mut());

        let stage = get_rbrb_stage(self);
        stage.get_inputs = Some(get_inputs);
        stage.parse_inputs = Some(parse_inputs);

        self
    }

    fn update_rollback_schedule(&mut self, f: impl FnOnce(&mut Schedule)) -> &mut Self {
        f(&mut get_rbrb_stage(self).schedule);
        self
    }
}

fn get_rbrb_stage(builder: &mut AppBuilder) -> &mut RbrbStage {
    builder
        .app
        .schedule
        .get_stage_mut::<RbrbStage>(&"rbrb_update")
        .expect("could not find RbrbStage, install RbrbPlugin")
}

fn serialize_inputs<I: serde::Serialize>(input: In<I>) -> Vec<u8> {
    bincode::serialize(&input.0).unwrap()
}

fn parse_inputs<I: serde::de::DeserializeOwned + Send + Sync + 'static>(world: &mut World) {
    let player_inputs = world
        .get_resource::<PlayerInputs>()
        .expect("should have specified PlayerInputs");
    let parsed_inputs = player_inputs
        .clone()
        .deep_map(|i| bincode::deserialize::<I>(&i).unwrap());
    world.insert_resource(parsed_inputs);
}
