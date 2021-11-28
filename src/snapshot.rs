use bevy_ecs::prelude::*;

#[derive(Default)]
pub struct Snapshotter {}

impl Snapshotter {
    pub fn save_to(&mut self, _vec: &mut Vec<u8>, _world: &mut World) {
        log::warn!("Unimplemented: save_to");
    }

    pub fn load_from(&mut self, _slice: &[u8], _world: &mut World) {
        log::warn!("Unimplemented: load_from");
    }
}
