use bevy_ecs::{prelude::*, system::ExclusiveSystem};
use rbrb::*;

use std::ops::ControlFlow;

pub struct RbrbStage {
    pub schedule: Schedule,
    pub get_inputs: Option<Box<dyn System<In = (), Out = Vec<u8>>>>,
    pub parse_inputs: Option<Box<dyn ExclusiveSystem>>,
    pub snapshotter: crate::snapshot::Snapshotter,
}

impl RbrbStage {
    pub fn new() -> Self {
        RbrbStage {
            schedule: Schedule::default(),
            get_inputs: None,
            parse_inputs: None,
            snapshotter: Default::default(),
        }
    }

    fn handle_request(&mut self, request: Request, world: &mut World) {
        match request {
            Request::CaptureLocalInput(vec) => {
                let inputs = self
                    .get_inputs
                    .as_mut()
                    .expect("no input system provided")
                    .run((), world);
                *vec = inputs;
            }
            Request::Advance {
                inputs,
                amount,
                confirmed,
                ..
            } => {
                world.insert_resource(inputs);
                world.insert_resource(crate::RbrbTime { delta: amount });
                world.insert_resource(confirmed);

                if let Some(s) = self.parse_inputs.as_mut() {
                    s.run(world);
                }
                self.schedule.run_once(world);

                world.remove_resource::<rbrb::Confirmation>();
                world.remove_resource::<crate::RbrbTime>();
                world.remove_resource::<PlayerInputs>();
            }

            Request::SaveTo(vec) => self.snapshotter.save_to(vec, world),
            Request::LoadFrom(slice) => self.snapshotter.load_from(slice, world),

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
        while let ControlFlow::Continue(()) = session.next_request(|request: Request<'_>| {
            self.handle_request(request, world);
        }) {}
        world.insert_resource(session);
    }
}
