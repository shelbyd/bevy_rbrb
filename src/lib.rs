use bevy_app::*;
use bevy_ecs::prelude::*;

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

        let mut parse_inputs = Box::new(parse_inputs::<I>.system());
        parse_inputs.initialize(self.world_mut());

        let stage = self
            .app
            .schedule
            .get_stage_mut::<RbrbStage>(&"rbrb_update")
            .expect("could not find RbrbStage, install RbrbPlugin");

        stage.get_inputs = Some(get_inputs);
        stage.parse_inputs = Some(parse_inputs);

        self
    }
}

fn serialize_inputs<I: serde::Serialize>(input: In<I>) -> Vec<u8> {
    bincode::serialize(&input.0).unwrap()
}

fn parse_inputs<I: serde::de::DeserializeOwned + Send + Sync + 'static>(
    player_inputs: In<PlayerInputs>,
    mut commands: Commands,
) {
    commands.insert_resource(
        player_inputs
            .0
            .deep_map(|i| bincode::deserialize::<I>(&i).unwrap()),
    );
}
