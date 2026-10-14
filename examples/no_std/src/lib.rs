//! A minimal `no_std` library that drives a [`Reactor`].
//!
//! The test at the bottom runs the reactor on the host, with `bevy_flurx` built without `std`.

#![no_std]

// The test harness needs `std`; the library itself does not.
#[cfg(test)]
extern crate std;

use bevy::prelude::*;
use bevy_flurx::prelude::*;

/// Incremented by the reactor every frame until it reaches 3.
#[derive(Resource, Default)]
pub struct Count(pub usize);

/// Set to `true` once the reactor has run to completion.
#[derive(Resource, Default)]
pub struct Finished(pub bool);

/// Spawns a reactor that waits, delays and then marks itself [`Finished`].
pub struct NoStdExamplePlugin;

impl Plugin for NoStdExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Count>()
            .init_resource::<Finished>()
            .add_systems(Startup, spawn_reactor);
    }
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(
            Update,
            wait::until(|mut count: ResMut<Count>| {
                count.0 += 1;
                count.0 == 3
            }),
        )
        .await;
        task.will(Update, delay::frames().with(2)).await;
        task.will(
            Update,
            once::run(|mut finished: ResMut<Finished>| {
                finished.0 = true;
            }),
        )
        .await;
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactor_runs_to_completion() {
        let mut app = App::new();
        app.add_plugins((FlurxPlugin, NoStdExamplePlugin));

        for _ in 0..10 {
            app.update();
        }

        assert_eq!(app.world().resource::<Count>().0, 3);
        assert!(app.world().resource::<Finished>().0);
    }
}
