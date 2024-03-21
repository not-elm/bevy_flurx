use std::marker::PhantomData;

use bevy::prelude::{IntoSystem, System, World};
use crate::selector::condition::wait::output;

use crate::selector::runner::{ReactorSystemOutput, RunReactor};

pub(crate) struct StandardReactorRunner<Sys, In, Out> {
    system: Sys,
    input: In,
    _m: PhantomData<(In, Out)>,
}


impl<Sys, In, Out> StandardReactorRunner<Sys, In, Out> {
    pub const fn new(
        system: Sys,
        input: In,
    ) -> StandardReactorRunner<Sys, In, Out> {
        Self {
            system,
            input,
            _m: PhantomData,
        }
    }
}

impl<Sys, In, Out> RunReactor for StandardReactorRunner<Sys, In, Option<Out>>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
       
        let out = self.system.run(self.input.clone(), world);
        self.system.apply_deferred(world);
        if let Some(output) = out {
            world.init_non_send_resource::<ReactorSystemOutput<Out>>();
            let mut map = world.remove_non_send_resource::<ReactorSystemOutput<Out>>().unwrap();
            map.push(self.system.system_type_id(), output);
            world.insert_non_send_resource(map);
            true
        } else {
            false
        }
    }
}