use bevy_ecs::prelude::*;
use rbrb::*;

use std::ops::ControlFlow;

pub struct RbrbStage {
    schedule: Schedule,
    pub get_inputs: Option<Box<dyn System<In = PlayerId, Out = Vec<u8>>>>,
    pub parse_inputs: Option<Box<dyn System<In = PlayerInputs, Out = ()>>>,
}

impl RbrbStage {
    pub fn new() -> Self {
        RbrbStage {
            schedule: Default::default(),
            get_inputs: None,
            parse_inputs: None,
        }
    }

    fn handle_request(&mut self, request: Request, world: &mut World, local_id: PlayerId) {
        match request {
            Request::CaptureLocalInput(vec) => {
                let inputs = self
                    .get_inputs
                    .as_mut()
                    .expect("no input system provided")
                    .run(local_id, world);
                *vec = inputs;
            }
            Request::Advance { inputs, .. } => {
                if let Some(s) = self.parse_inputs.as_mut() {
                    s.run(inputs.clone(), world);
                }
                world.insert_resource(inputs);
                self.schedule.run_once(world);
                world.remove_resource::<PlayerInputs>();
            }

            Request::SaveTo(vec) => {
                crate::snapshot::save_to(vec, world);
            }

            unhandled => {
                unimplemented!("unhandled: {:?}", unhandled);
            }
        }
    }
}

impl Stage for RbrbStage {
    fn run(&mut self, world: &mut World) {
        let mut session = match world.remove_resource::<Session>() {
            Some(s) => s,
            None => return,
        };
        let local_id = session.local_player_id();
        while let ControlFlow::Continue(()) = session.next_request(|request: Request<'_>| {
            self.handle_request(request, world, local_id);
        }) {}
        world.insert_resource(session);
    }
}
