//! [`wait::event`] creates a task related to waiting to receive events.
//!
//! - [`wait::event::comes`]
//! - [`wait::event::read`]

use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Event, Events, Local, ResMut};

use crate::prelude::seed::ActionSeed;
use crate::prelude::wait;

/// Waits until the specified event is sent
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::comes::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn comes<E>() -> ActionSeed
    where
        E: Event,
{
    wait::until(|mut er: Local<Option<ManualEventReader<E>>>,
                 mut events: ResMut<Events<E>>| {
        if er.is_none() {
            if 0 < events.iter_current_update_events().count() {
                events.clear();
                return true;
            }
            er.replace(events.get_reader_current());
        }

        if 0 < er.as_mut().unwrap().read(&events).count() {
            events.clear();
            true
        } else {
            false
        }
    })
}

/// Waits until the specified event is sent.
///
/// This is similar to [`wait::event::comes`], except that it returns the event itself.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::read::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn read<E>() -> ActionSeed<(), E>
    where
        E: Event + Clone,
{
    wait::output(|mut er: Local<Option<ManualEventReader<E>>>,
                  mut events: ResMut<Events<E>>| {
        if er.is_none() {
            if let Some(event) = events.iter_current_update_events().last().cloned() {
                events.clear();
                return Some(event);
            }
            er.replace(events.get_reader_current());
        }
        if let Some(event) = er.as_mut().unwrap().read(&events).last().cloned() {
            events.clear();
            Some(event)
        } else {
            None
        }
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, EventWriter, In};
    use bevy_test_helper::event::{DirectEvents, TestEvent1, TestEvent2};

    use crate::action::{once, wait};
    use crate::prelude::{Either, Pipe, Reactor, Then};
    use crate::tests::test_app;

    #[test]
    fn wait_until_event_consumed_events() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::event::send_default::<TestEvent1>()
                        .then(wait::event::comes::<TestEvent1>()),
                )
                    .await;

                task.will(Update, {
                    wait::either(wait::event::comes::<TestEvent1>(), once::run(|| {})).pipe(
                        once::run(
                            |In(either): In<Either<(), ()>>, mut ew: EventWriter<TestEvent2>| {
                                if either.is_right() {
                                    ew.send_default();
                                }
                            },
                        ),
                    )
                })
                    .await;
            }));
        });

        app.update();
        app.update();

        let mut er = ManualEventReader::<TestEvent2>::default();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn wait_read_event_consumed_events() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::event::send_default::<TestEvent1>()
                        .then(wait::event::read::<TestEvent1>()),
                )
                    .await;

                task.will(Update, {
                    wait::either(wait::event::read::<TestEvent1>(), once::run(|| {})).pipe(
                        once::run(
                            |In(either): In<Either<TestEvent1, ()>>, mut ew: EventWriter<TestEvent2>| {
                                if either.is_right() {
                                    ew.send_default();
                                }
                            },
                        ),
                    )
                })
                    .await;
            }));
        });

        app.update();
        app.update();

        let mut er = ManualEventReader::<TestEvent2>::default();
        app.assert_event_comes(&mut er);
    }
}
