//! A switch is a structure that represents two states: `on` and `off`.
//! 
//! This is to solve the problem that systems created from `Reactors`
//! cannot run except on the main thread.


use std::marker::PhantomData;

use bevy::prelude::{FromWorld, Mut, Res, ResMut, Resource, Schedules, World};

use crate::AfterLast;
use crate::runner::initialize_schedule;

/// A Condition-satisfying system that returns true if the switch has been turned on. 
#[inline]
pub fn switch_turned_on<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.turned_on())
}

/// A Condition-satisfying system that returns true if the switch has just been turned on. 
#[inline]
pub fn switch_just_turned_on<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.just_turned_on())
}

/// A Condition-satisfying system that returns true if the switch has been turned off. 
#[inline]
pub fn switch_turned_off<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.turned_off())
}

/// A Condition-satisfying system that returns true if the switch has just been turned off. 
#[inline]
pub fn switch_just_turned_off<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.just_turned_off())
}

/// A switch is a structure that represents two states: `on` and `off`.
/// 
/// This is to solve the problem that systems created from `Reactors`
/// cannot run except on the main thread.
/// 
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// 
/// struct HeavyTask;
/// 
/// App::new()
///     .add_systems(Update, (|mut switch: ResMut<Switch<HeavyTask>>|{
///         // heavy task
///         //..
///         
///         switch.off();
///     }).run_if(switch_turned_on::<HeavyTask>))
///     .add_systems(Update, |world: &mut World|{
///         world.schedule_reactor(|task| async move{
///             task.will(Update, once::switch::on::<HeavyTask>()).await;
///             task.will(Update, wait::switch::off::<HeavyTask>()).await;
///         }); 
///     });
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Switch<M> {
    just_change: bool,
    turned_on: bool,
    _m: PhantomData<M>,
}

impl<M> Resource for Switch<M>
    where M: Send + Sync + 'static
{}

impl<M> Switch<M>
    where M: Send + Sync + 'static
{
    /// Create new Switch with initial status.
    #[inline(always)]
    const fn new(turn_on: bool) -> Switch<M> {
        Self {
            just_change: true,
            turned_on: turn_on,
            _m: PhantomData,
        }
    }

    /// Returns true if the switch has just been turned on.
    #[inline(always)]
    pub const fn just_turned_on(&self) -> bool {
        self.just_change && self.turned_on
    }

    /// Returns true if the switch has just been turned off.
    #[inline(always)]
    pub const fn just_turned_off(&self) -> bool {
        self.just_change && self.turned_off()
    }

    /// Returns true if switch is on.
    #[inline(always)]
    pub const fn turned_on(&self) -> bool {
        self.turned_on
    }

    /// Returns true if switch is off.
    #[inline(always)]
    pub const fn turned_off(&self) -> bool {
        !self.turned_on
    }

    /// Sets turn on or off.
    pub fn set(&mut self, turn_on: bool) {
        if turn_on {
            self.on();
        } else {
            self.off();
        }
    }

    /// Turn on the switch.
    #[inline(always)]
    pub fn on(&mut self) {
        if self.turned_off() {
            self.just_change = true;
            self.turned_on = true;
        }
    }

    /// Turn off the switch.
    #[inline(always)]
    pub fn off(&mut self) {
        if self.turned_on {
            self.just_change = true;
            self.turned_on = false;
        }
    }

    pub(crate) fn setup(world: &mut World, turn_on: bool) -> Mut<Switch<M>> {
        if world.get_resource::<Switch<M>>().is_some() {
            let mut s = world.resource_mut::<Switch<M>>();
            s.set(turn_on);
            s
        } else {
            let mut s = Self::from_world(world);
            s.set(turn_on);
            world.insert_resource(s);
            world.resource_mut::<Switch<M>>()
        }
    }
}

impl<M> FromWorld for Switch<M>
    where M: Send + Sync + 'static
{
    fn from_world(world: &mut World) -> Self {
        world.resource_scope(|_, mut schedules: Mut<Schedules>| {
            let schedule = initialize_schedule(&mut schedules, AfterLast);
            schedule.add_systems(|mut switch: ResMut<Switch<M>>| {
                switch.just_change = false;
            });
        });

        Self::new(false)
    }
}


#[cfg(test)]
mod tests {
    use bevy::prelude::{Res, ResMut, Update};
    use bevy_test_helper::resource::bool::{Bool, BoolExtension};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::prelude::Switch;
    use crate::tests::test_app;

    struct T;

    #[test]
    fn off() {
        let mut s = Switch::<T>::new(true);
        assert!(s.just_change);
        assert!(s.turned_on);
        s.just_change = false;
        s.off();
        assert!(s.just_change);
        assert!(s.turned_off());
    }

    #[test]
    fn on() {
        let mut s = Switch::<T>::new(false);
        assert!(s.just_change);
        assert!(s.turned_off());
        s.just_change = false;
        s.on();
        assert!(s.just_change);
        assert!(s.turned_on());
    }

    #[test]
    fn cleanup() {
        let mut app = test_app();
        app.init_resource::<Switch<T>>();
        app.add_systems(Update, |mut b: ResMut<Bool>, s: Res<Switch<T>>| {
            if s.just_turned_off() {
                **b = true;
            }
        });
        app.update();
        assert!(app.is_bool_true());
        assert!(!app.resource::<Switch<T>>().just_turned_off());
    }
}