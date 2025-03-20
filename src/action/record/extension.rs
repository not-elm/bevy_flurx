//! Allows undo and redo requests to be made using [`RequestUndo`] and [`RequestRedo`]
//! from outside [`Reactor`].

use crate::action::record;
use crate::prelude::{Omit, Reactor, Then};
use bevy::app::{App, PostUpdate, Update};
use bevy::prelude::{Commands, Event, EventReader};

/// Represents a request `undo` operations.
///
/// If an undo or redo is already in progress, the request will be ignored.
#[derive(Event, Eq, PartialEq, Debug)]
pub enum RequestUndo<Act> {
    /// [`record::undo::once`]
    Once,

    /// [`record::undo::index_to`]
    IndexTo(usize),

    /// [`record::undo::to`]
    To(Act),

    /// [`record::undo::all`]
    All,
}

/// Represents a request `redo` operations.
///
/// If an undo or redo is already in progress, the request will be ignored.
#[derive(Event, Eq, PartialEq, Debug)]
pub enum RequestRedo<Act> {
    /// [`record::redo::once`]
    Once,

    /// [`record::redo::index_to`]
    IndexTo(usize),

    /// [`record::redo::to`]
    To(Act),

    /// [`record::redo::all`]
    All,
}

/// Allows undo and redo requests to be made using [`RequestUndo`] and [`RequestRedo`]
/// from outside [`Reactor`].
pub trait RecordExtension {
    /// Set up [`RequestUndo`] and [`RequestRedo`] and their associated systems.
    fn add_record_events<Act>(&mut self) -> &mut Self
    where
        Act: Clone + PartialEq + Send + Sync + 'static;
}

impl RecordExtension for App {
    fn add_record_events<Act>(&mut self) -> &mut Self
    where
        Act: Clone + PartialEq + Send + Sync + 'static,
    {
        self
            .add_event::<RequestUndo<Act>>()
            .add_event::<RequestRedo<Act>>()
            .add_systems(PostUpdate, (request_undo::<Act>, request_redo::<Act>))
    }
}

fn request_undo<Act>(mut commands: Commands, mut er: EventReader<RequestUndo<Act>>)
where
    Act: Clone + Send + PartialEq + Sync + 'static,
{
    if let Some(actions) = er
        .read()
        .map(|req| match req {
            RequestUndo::To(act) => record::undo::to().with(act.clone()).omit(),
            RequestUndo::IndexTo(i) => record::undo::index_to::<Act>().with(*i).omit(),
            RequestUndo::Once => record::undo::once::<Act>().omit(),
            RequestUndo::All => record::undo::all::<Act>().omit(),
        })
        .reduce(|r1, r2| r1.then(r2))
    {
        commands.spawn(Reactor::schedule(|task| async move {
            task.will(Update, actions).await;
        }));
    }
}

fn request_redo<Act>(mut commands: Commands, mut er: EventReader<RequestRedo<Act>>)
where
    Act: Clone + Send + PartialEq + Sync + 'static,
{
    if let Some(actions) = er
        .read()
        .map(|req| match req {
            RequestRedo::To(act) => record::redo::to().with(act.clone()).omit(),
            RequestRedo::IndexTo(i) => record::redo::index_to::<Act>().with(*i).omit(),
            RequestRedo::Once => record::redo::once::<Act>().omit(),
            RequestRedo::All => record::redo::all::<Act>().omit(),
        })
        .reduce(|r1, r2| r1.then(r2))
    {
        commands.spawn(Reactor::schedule(|task| async move {
            task.will(Update, actions).await;
        }));
    }
}

//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, IntoScheduleConfigs};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::record::tests::push_undo_increment;
    use crate::prelude::record::tests::push_num_act;
    use crate::prelude::{Reactor, RequestRedo, RequestUndo, Then};
    use crate::tests::{test_app, NumAct, TestAct};

    #[test]
    fn test_request_undo_once() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push_undo_increment()).await.unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.write(RequestUndo::Once);
        });
        app.update();
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn test_request_undo_index_to() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment()),
                )
                    .await
                    .unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.write(RequestUndo::IndexTo(1));
        });
        app.update();
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn test_request_undo_to() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_num_act(0).then(push_num_act(1)).then(push_num_act(2)),
                )
                    .await;
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<NumAct>>| {
            ew.write(RequestUndo::To(NumAct(1)));
        });
        app.update();
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn test_request_undo_all() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment()),
                )
                    .await
                    .unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.write(RequestUndo::All);
        });
        app.update();
        app.update();
        app.assert_resource_eq(Count(3));
    }

    #[test]
    fn test_request_redo_once() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment()),
                )
                    .await
                    .unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.write(RequestUndo::All);
        });
        app.update();
        app.world_mut()
            .run_system_once(|mut ew: EventWriter<RequestRedo<TestAct>>| {
                ew.write(RequestRedo::Once);
            })
            .expect("Failed to run system");
        app.update();
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn test_request_redo_index_to() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment()),
                )
                    .await
                    .unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.write(RequestUndo::All);
        });
        app.update();
        app.world_mut()
            .run_system_once(|mut ew: EventWriter<RequestRedo<TestAct>>| {
                ew.write(RequestRedo::IndexTo(1));
            })
            .expect("Failed to run system");
        app.update();
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn test_request_redo_to() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_num_act(0).then(push_num_act(1)).then(push_num_act(2)),
                )
                    .await;
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<NumAct>>| {
            ew.write(RequestUndo::All);
        });
        app.update();
        app.world_mut()
            .run_system_once(|mut ew: EventWriter<RequestRedo<NumAct>>| {
                ew.write(RequestRedo::To(NumAct(1)));
            })
            .expect("Failed to run system");
        app.update();
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn test_request_redo_all() {
        let mut app = test_app();

        app.add_systems(Startup, (
            |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(
                        Update,
                        push_num_act(0).then(push_num_act(1)).then(push_num_act(2)),
                    )
                        .await;
                }));
            },
            |mut ew: EventWriter<RequestUndo<NumAct>>| {
                ew.write(RequestUndo::All);
            }
        ).chain());
        app.update();
        app.world_mut()
            .run_system_once(|mut ew: EventWriter<RequestRedo<NumAct>>| {
                ew.write(RequestRedo::All);
            })
            .expect("Failed to run system");
        app.update();
        app.update();
        app.assert_resource_eq(Count(0));
    }
}
