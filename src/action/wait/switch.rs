use bevy::prelude::Res;

use crate::action::{wait, with};
use crate::action::switch::Switch;
use crate::prelude::TaskAction;

#[inline]
pub fn on<M: Send + Sync + 'static>() -> impl TaskAction<In=(), Out=()> {
    with((), wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.is_on())
    }))
}

#[inline]
pub fn off<M: Send + Sync + 'static>() -> impl TaskAction<In=(), Out=()> {
    with((), wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.is_off())
    }))
}