use std::marker::PhantomData;

use bevy::prelude::{IntoSystem, System, World};

use crate::runner::{TaskOutputMap, RunTask};

pub(crate) struct MultiTimesRunner<Sys, In, Out> {
    system: Sys,
    input: In,
    _m: PhantomData<Out>,
}

impl<Sys, In, Out> MultiTimesRunner<Sys, In, Out> {
    #[inline]
    pub const fn new(
        system: Sys,
        input: In,
    ) -> MultiTimesRunner<Sys, In, Out> {
        Self {
            system,
            input,
            _m: PhantomData,
        }
    }
}

impl<Sys, In, Out> RunTask for MultiTimesRunner<Sys, In, Option<Out>>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        let out = self.system.run(self.input.clone(), world);
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