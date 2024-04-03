use std::marker::PhantomData;

use bevy::prelude::{IntoSystem, System, World};

use crate::runner::{TaskOutputMap, RunTask};

pub(crate) struct OnceRunner<Sys, In, Out> {
    system: Sys,
    input: Option<In>,
    _m: PhantomData<Out>,
}

impl<Sys, In, Out> OnceRunner<Sys, In, Out> {
    #[inline]
    pub const fn new(
        system: Sys,
        input: In,
    ) -> OnceRunner<Sys, In, Out> {
        Self {
            system,
            input: Some(input),
            _m: PhantomData,
        }
    }
}

impl<Sys, In, Out> RunTask for OnceRunner<Sys, In, Option<Out>>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        let Some(input) = self.input.take() else{
            return true;
        };
        let out = self.system.run(input, world);
        self.system.apply_deferred(world);
        if let Some(output) = out {
            world.init_non_send_resource::<TaskOutputMap<Out>>();
            let mut map = world.remove_non_send_resource::<TaskOutputMap<Out>>().unwrap();
            map.push(self.system.system_type_id(), output);
            world.insert_non_send_resource(map);
            true
        } else {
            false
        }
    }
}