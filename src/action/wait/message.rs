//! [`wait::message`] creates a task related to waiting to receive messages.

use crate::prelude::seed::ActionSeed;
use crate::prelude::wait;
use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;

/// Waits until the message is received.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::message::comes::<AppExit>()).await;
/// });
/// ```
pub fn comes<M>() -> ActionSeed
where
    M: Message,
{
    wait::until(
        |mut er: Local<Option<MessageCursor<M>>>, mut messages: ResMut<Messages<M>>| {
            if er.is_none() {
                if 0 < messages.iter_current_update_messages().count() {
                    messages.clear();
                    return true;
                }
                er.replace(messages.get_cursor_current());
            }

            if 0 < er.as_mut().unwrap().read(&messages).count() {
                messages.clear();
                true
            } else {
                false
            }
        },
    )
}

/// Waits until the message is received and the message matches the predicate.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::message::comes_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });
/// ```
pub fn comes_and<M>(predicate: impl Fn(&M) -> bool + Send + Sync + 'static) -> ActionSeed
where
    M: Message,
{
    wait::until(
        move |mut er: Local<Option<MessageCursor<M>>>, mut messages: ResMut<Messages<M>>| {
            if er.is_none() {
                let received = messages.iter_current_update_messages().any(&predicate);
                if received {
                    messages.clear();
                    return true;
                }
                er.replace(messages.get_cursor_current());
            }

            if er.as_mut().unwrap().read(&messages).any(&predicate) {
                messages.clear();
                true
            } else {
                false
            }
        },
    )
}

/// Waits until the message is received.
///
/// This is similar to [`wait::message::comes`], but it returns a cloned message.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::message::read::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn read<M>() -> ActionSeed<(), M>
where
    M: Message + Clone,
{
    wait::message::read_and(|_| true)
}

/// Waits until the message is received and the message matches the predicate.
///
/// This is similar to [`wait::message::comes`], but it returns a cloned message.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::message::read_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn read_and<M>(predicate: impl Fn(&M) -> bool + Send + Sync + 'static) -> ActionSeed<(), M>
where
    M: Message + Clone,
{
    wait::output(
        move |mut er: Local<Option<MessageCursor<M>>>, mut messages: ResMut<Messages<M>>| {
            if er.is_none() {
                let message = {
                    messages
                        .iter_current_update_messages()
                        .find(|e| predicate(e))
                        .cloned()
                };
                if let Some(message) = message {
                    messages.clear();
                    return Some(message);
                }
            }
            let er = er.get_or_insert_with(|| messages.get_cursor_current());
            if let Some(message) = er.read(&messages).find(|e| predicate(e)).cloned() {
                messages.clear();
                Some(message)
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
    use bevy::prelude::*;
    use bevy_test_helper::event::*;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn wait_until_message_consumed_messages() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::message::write_default::<TestEvent1>()
                        .then(wait::message::comes::<TestEvent1>()),
                )
                .await;

                task.will(Update, {
                    wait::either(wait::message::comes::<TestEvent1>(), once::run(|| {})).pipe(
                        once::run(
                            |In(either): In<Either<(), ()>>, mut ew: MessageWriter<TestEvent2>| {
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

        let mut er = app.resource_mut::<Messages<TestEvent2>>().get_cursor();
        app.assert_message_comes(&mut er);
    }

    #[test]
    fn wait_read_message_consumed_messages() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::message::write_default::<TestEvent1>()
                        .then(wait::message::read::<TestEvent1>()),
                )
                .await;

                task.will(Update, {
                    wait::either(wait::message::read::<TestEvent1>(), once::run(|| {})).pipe(
                        once::run(
                            |In(either): In<Either<TestEvent1, ()>>,
                             mut ew: MessageWriter<TestEvent2>| {
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

        let mut er = app.resource_mut::<Messages<TestEvent2>>().get_cursor();
        app.assert_message_comes(&mut er);
    }

    #[derive(Message, Clone, Debug, Eq, PartialEq, Resource)]
    struct PredicateMessage(bool);

    #[test]
    fn wait_read_message_with_predicate() {
        let mut app = test_app();
        app.add_message::<PredicateMessage>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    wait::message::read_and::<PredicateMessage>(|e| e.0).pipe(once::res::insert()),
                )
                .await;
            }));
        });
        assert_message_predicate(&mut app);
    }

    #[test]
    fn wait_comes_message_with_predicate() {
        let mut app = test_app();
        app.add_message::<PredicateMessage>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    wait::message::comes_and::<PredicateMessage>(|e| e.0)
                        .then(once::res::insert().with(PredicateMessage(true))),
                )
                .await;
            }));
        });
        assert_message_predicate(&mut app);
    }

    fn assert_message_predicate(app: &mut App) {
        app.update();
        assert!(!app.world().contains_resource::<PredicateMessage>());

        app.write(PredicateMessage(false));
        app.update();
        assert!(!app.world().contains_resource::<PredicateMessage>());

        app.write(PredicateMessage(true));
        app.update();
        app.assert_resource_eq(PredicateMessage(true));
    }
}
