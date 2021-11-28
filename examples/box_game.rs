use bevy::prelude::*;
use serde::*;
use std::{net::SocketAddr, time::Duration};
use structopt::*;

use bevy_rbrb::{
    BasicUdpSocket, Confirmed, PlayerId, PlayerInputs, RbrbAppExt, RbrbPlugin, SessionBuilder,
};

#[derive(StructOpt)]
struct Options {
    #[structopt(long)]
    local_port: u16,

    #[structopt(long)]
    local_index: u16,

    remote_players: Vec<SocketAddr>,
}

#[derive(Default, Deserialize, Serialize)]
struct BoxGameInput {
    direction: Vec2,
}

fn main() {
    let options = Options::from_args();

    let session = SessionBuilder::default()
        .remote_players(&options.remote_players)
        .local_player(options.local_index)
        .step_size(Duration::from_millis(10))
        // TODO(shelbyd): Don't specify default input with typed input system. Requires matching
        // internal serialization.
        .default_inputs(bincode::serialize(&BoxGameInput::default()).unwrap())
        .with_socket(BasicUdpSocket::bind(options.local_port).unwrap())
        .start()
        .unwrap();

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RbrbPlugin)
        .with_session(session)
        .with_typed_input_system(capture_input.system())
        .update_rollback_schedule(|sched| {
            sched
                .add_stage("box_game", SystemStage::parallel())
                .add_system_to_stage("box_game", move_boxes.system());
        })
        .run()
}

fn capture_input(_local_player_id: In<PlayerId>, keyboard: Res<Input<KeyCode>>) -> BoxGameInput {
    let mut input = BoxGameInput::default();
    if keyboard.pressed(KeyCode::Up) {
        input.direction.y -= 1.;
    }
    if keyboard.pressed(KeyCode::Down) {
        input.direction.y += 1.;
    }
    if keyboard.pressed(KeyCode::Left) {
        input.direction.x -= 1.;
    }
    if keyboard.pressed(KeyCode::Right) {
        input.direction.x += 1.;
    }
    input
}

fn move_boxes(
    inputs: Res<PlayerInputs<Confirmed<BoxGameInput>>>,
    mut boxes: Query<(&mut Transform, &PlayerId)>,
) {
    for (mut xform, player) in boxes.iter_mut() {
        let input = inputs.get(player).expect("should have inputs for all players");
        xform.translation += input.as_inner().direction.clamp_length_max(1.).extend(0.);
    }
}
