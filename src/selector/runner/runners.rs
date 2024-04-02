use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;

use crate::selector::runner::RunReactor;

pub(super) struct ReactorRunners<Label> {
    pub(super) systems: Vec<Box<dyn RunReactor>>,
    _m: PhantomData<Label>,
}

impl<Label> Default for ReactorRunners<Label>
    where Label: ScheduleLabel
{
    #[inline]
    fn default() -> ReactorRunners<Label> {
        ReactorRunners {
            systems: Vec::new(),
            _m: PhantomData,
        }
    }
}

impl<Label> ReactorRunners<Label>
    where Label: ScheduleLabel
{
    pub(crate) fn run(&mut self, world: &mut World) -> bool {
        let mut pending = Vec::with_capacity(self.systems.len());
        while let Some(mut runner) = self.systems.pop() {
            if !runner.run(world) {
                pending.push(runner);
            }
        }
        if pending.is_empty() {
            true
        } else {
            self.systems = pending;
            false
        }
    }
}