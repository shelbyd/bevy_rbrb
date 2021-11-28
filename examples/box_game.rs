use bevy::prelude::*;
use serde::*;
use std::{net::SocketAddr, time::Duration};
use structopt::*;

use bevy_rbrb::{
    BasicUdpSocket, Confirmed, PlayerId, PlayerInputs, RbrbAppExt, RbrbPlugin, RbrbTime, Session,
    SessionBuilder,
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
        .add_startup_system(spawn_players.system())
        .with_typed_input_system(capture_input.system())
        .update_rollback_schedule(|sched| {
            sched
                .add_stage("box_game", SystemStage::parallel())
                .add_system_to_stage("box_game", move_boxes.system());
        })
        .run()
}

struct Player {
    id: PlayerId,
}

fn spawn_players(
    session: Res<Session>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let floor_size = 5.;
    let cube_size = 0.2;

    let _floor = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: floor_size })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.2).into()),
        ..Default::default()
    });

    let player_count = session.players().count();
    for (id, _player) in session.players() {
        let angle = id as f32 / player_count as f32 * 2. * std::f32::consts::PI;
        let radius = floor_size / 4.;

        let mut transform = Transform::default();
        transform.translation.x = radius * angle.cos();
        transform.translation.y = cube_size / 2.;
        transform.translation.z = radius * angle.sin();

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                material: materials.add(Color::rgb(0., 0., 1.).into()),
                transform,
                ..Default::default()
            })
            .insert(Player { id });
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 7.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
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
    time: Res<RbrbTime>,
    mut boxes: Query<(&mut Transform, &Player)>,
) {
    let speed = 3.;

    for (mut xform, player) in boxes.iter_mut() {
        let input = inputs
            .get(&player.id)
            .expect("should have inputs for all players")
            .as_inner();
        let movement = input.direction.clamp_length_max(1.) * speed * time.delta.as_secs_f32();
        xform.translation += Vec3::new(movement.x, 0., movement.y);
    }
}
