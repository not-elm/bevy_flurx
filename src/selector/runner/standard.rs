use bevy::ecs::system::SystemId;
use bevy::prelude::World;

use crate::selector::runner::{ReactorSystemOutput, RunReactor};

pub(crate) struct StandardReactorRunner<In, Out> {
    system_id: SystemId<In, Out>,
    input: In,
}


impl<In, Out> StandardReactorRunner<In, Out> {
    pub const fn new(
        system_id: SystemId<In, Out>,
        input: In,
    ) -> StandardReactorRunner<In, Out> {
        Self {
            system_id,
            input,
        }
    }
}

impl<In, Out> RunReactor for StandardReactorRunner<In, Option<Out>>
    where 
        In: Clone + 'static,
        Out: 'static
{
    fn run(&self, world: &mut World) -> bool {
        if let Some(output) = world.run_system_with_input(self.system_id, self.input.clone()).expect("failed run reactor system") {
            world.init_non_send_resource::<ReactorSystemOutput<In, Out>>();
            let mut map = world.remove_non_send_resource::<ReactorSystemOutput<In, Out>>().unwrap();
            map.push(self.system_id, output);
            world.insert_non_send_resource(map);
            true
        } else {
            false
        }
    }
}