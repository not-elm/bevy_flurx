//! Allows undo and redo requests to be made using [`RequestUndo`] and [`RequestRedo`] 
//! from outside [`Reactor`].
//! 
//! events
//! - [`RequestUndo`]
//! - [`RequestRedo`]
//! 
//! traits
//! - [`RecordExtension`]


use bevy::app::{App, PostUpdate, Update};
use bevy::prelude::{Commands, Event, EventReader};

use crate::action::{Omit, record};
use crate::prelude::{Reactor, Then};

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
            Act: Clone + PartialEq + Send + Sync + 'static
    {
        self
            .add_event::<RequestUndo<Act>>()
            .add_event::<RequestRedo<Act>>()
            .add_systems(PostUpdate, (
                request_undo::<Act>,
                request_redo::<Act>
            ))
    }
}

fn request_undo<Act>(
    mut commands: Commands,
    mut er: EventReader<RequestUndo<Act>>,
)
    where
        Act: Clone + Send + PartialEq + Sync + 'static
{
    if let Some(actions) = er.read()
        .map(|req| match req {
            RequestUndo::To(act) => record::undo::to().with(act.clone()).omit(),
            RequestUndo::IndexTo(i) => record::undo::index_to::<Act>().with(*i).omit(),
            RequestUndo::Once => record::undo::once::<Act>().omit(),
            RequestUndo::All => record::undo::all::<Act>().omit()
        })
        .reduce(|r1, r2| {
            r1.then(r2)
        }) {
        er.clear();

        commands.spawn(Reactor::schedule(|task| async move {
            task.will(Update, actions).await;
        }));
    }
}

fn request_redo<Act>(
    mut commands: Commands,
    mut er: EventReader<RequestRedo<Act>>,
)
    where
        Act: Clone + Send + PartialEq + Sync + 'static
{
    if let Some(actions) = er.read()
        .map(|req| match req {
            RequestRedo::To(act) => record::redo::to().with(act.clone()).omit(),
            RequestRedo::IndexTo(i) => record::redo::index_to::<Act>().with(*i).omit(),
            RequestRedo::Once => record::redo::once::<Act>().omit(),
            RequestRedo::All => record::redo::all::<Act>().omit()
        })
        .reduce(|r1, r2| {
            r1.then(r2)
        }) {
        commands.spawn(Reactor::schedule(|task| async move {
            task.will(Update, actions).await;
        }));
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{PreStartup, Startup, Update};
    use bevy::prelude::{Commands, EventWriter};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::record::tests::push_undo_increment;
    use crate::prelude::{Reactor, RequestUndo};
    use crate::tests::{test_app, TestAct};

    #[test]
    fn request_undo() {
        let mut app = test_app();
        app.add_systems(PreStartup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push_undo_increment()).await.unwrap();
            }));
        });
        app.add_systems(Startup, |mut ew: EventWriter<RequestUndo<TestAct>>| {
            ew.send(RequestUndo::Once);
        });
        app.update();
        app.update();
        app.assert_resource_eq(Count(1));
    }
}