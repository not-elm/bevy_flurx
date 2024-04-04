use bevy::prelude::{Commands, Res, ResMut};

use crate::action::once;
use crate::action::switch::Switch;
use crate::prelude::TaskAction;

#[inline]
pub fn on<M>() -> impl TaskAction<In=(), Out=()>
    where M: Send + Sync + 'static
{
    once::run(|mut commands: Commands, switch: Option<ResMut<Switch<M>>>| {
        if let Some(mut switch) = switch {
            switch.on();
        } else {
            commands.insert_resource(Switch::<M>::new(true));
        }
    })
}

#[inline]
pub fn off<M>() -> impl TaskAction<In=(), Out=()>
    where M: Send + Sync + 'static
{
    once::run(|mut commands: Commands, switch: Option<ResMut<Switch<M>>>| {
        if let Some(mut switch) = switch {
            switch.off();
        } else {
            commands.insert_resource(Switch::<M>::new(false));
        }
    })
}

#[inline]
pub fn switch_on<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.is_on())
}



