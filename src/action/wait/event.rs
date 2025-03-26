//! [`wait::event`] creates a task related to waiting to receive events.

use crate::prelude::seed::ActionSeed;
use crate::prelude::wait;
use bevy::ecs::event::EventCursor;
use bevy::prelude::{Event, Events, Local, ResMut};

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
pub fn comes<E>() -> ActionSeed
where
    E: Event,
{
    wait::until(
        |mut er: Local<Option<EventCursor<E>>>, mut events: ResMut<Events<E>>| {
            if er.is_none() {
                if 0 < events.iter_current_update_events().count() {
                    events.clear();
                    return true;
                }
                er.replace(events.get_cursor_current());
            }

            if 0 < er.as_mut().unwrap().read(&events).count() {
                events.clear();
                true
            } else {
                false
            }
        },
    )
}

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
///     task.will(Update, wait::event::comes_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });
/// ```
pub fn comes_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed
where
    E: Event,
{
    wait::until(
       move  |mut er: Local<Option<EventCursor<E>>>, mut events: ResMut<Events<E>>| {
            if er.is_none() {
                let received = events
                    .iter_current_update_events()
                    .any(&predicate);
                if received {
                    events.clear();
                    return true;
                }
                er.replace(events.get_cursor_current());
            }

            if er.as_mut().unwrap().read(&events).any(&predicate) {
                events.clear();
                true
            } else {
                false
            }
        },
    )
}

/// Waits until the event is received.
///
/// This is similar to [`wait::event::comes`], but it returns a cloned event.
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
    wait::event::read_and(|_|true)
}

/// Waits until the event is received and the event matches the predicate.
/// 
/// This is similar to [`wait::event::comes`], but it returns a cloned event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::read_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });   
/// ```
#[inline(always)]
pub fn read_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed<(), E>
where
    E: Event + Clone,
{
    wait::output(
        move |mut er: Local<Option<EventCursor<E>>>, mut events: ResMut<Events<E>>| {
            if er.is_none() {
                let event = {
                    events
                        .iter_current_update_events()
                        .find(|e| predicate(e))
                        .cloned()
                };
                if let Some(event) = event {
                    events.clear();
                    return Some(event);
                }
            }
            let er = er.get_or_insert_with(|| {
                events.get_cursor_current()
            });
            if let Some(event) = er.read(&events).find(|e|predicate(e)).cloned() {
                events.clear();
                Some(event)
            } else {
                None
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::action::{once, wait};
    use crate::prelude::{Either, Pipe, Reactor, Then};
    use crate::tests::test_app;
    use bevy::app::{App, Startup, Update};
    use bevy::prelude::{Commands, Event, EventWriter, Events, In, Resource};
    use bevy_test_helper::event::{DirectEvents, TestEvent1, TestEvent2};
    use bevy_test_helper::resource::DirectResourceControl;

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
                                    ew.write_default();
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

        let mut er = app.resource_mut::<Events<TestEvent2>>().get_cursor();
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
                            |In(either): In<Either<TestEvent1, ()>>,
                             mut ew: EventWriter<TestEvent2>| {
                                if either.is_right() {
                                    ew.write_default();
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

        let mut er = app.resource_mut::<Events<TestEvent2>>().get_cursor();
        app.assert_event_comes(&mut er);
    }


    #[derive(Event, Clone, Debug, Eq, PartialEq, Resource)]
    struct PredicateEvent(bool);

    #[test]
    fn wait_read_event_with_predicate(){
        let mut app = test_app();
        app.add_event::<PredicateEvent>();
        app.add_systems(Startup, |mut commands: Commands|{
           commands.spawn(Reactor::schedule(|task|async move{
               task.will(Update, wait::event::read_and::<PredicateEvent>(|e|e.0)
                   .pipe(once::res::insert())
               ).await;
           }));
        });
        assert_event_predicate(&mut app);
    }

    #[test]
    fn wait_comes_event_with_predicate(){
        let mut app = test_app();
        app.add_event::<PredicateEvent>();
        app.add_systems(Startup, |mut commands: Commands|{
            commands.spawn(Reactor::schedule(|task|async move{
                task.will(Update, wait::event::comes_and::<PredicateEvent>(|e|e.0)
                    .then(once::res::insert().with(PredicateEvent(true)))
                ).await;
            }));
        });
        assert_event_predicate(&mut app);
    }

    fn assert_event_predicate(app: &mut App){
        app.update();
        assert!(! app.world().contains_resource::<PredicateEvent>());

        app.send(PredicateEvent(false));
        app.update();
        assert!(!app.world().contains_resource::<PredicateEvent>());

        app.send(PredicateEvent(true));
        app.update();
        app.assert_resource_eq(PredicateEvent(true));
    }
}
