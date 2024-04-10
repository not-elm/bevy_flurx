//! Provides the mechanism to pipe the actions. 
//!
//! trait
//!
//! [`Pipe`]


use bevy::prelude::World;

use crate::action::remake::Remake;
use crate::prelude::seed::ActionSeed;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

/// Provides the mechanism to pipe the actions.
pub trait Pipe<I1, O1, O2, A> {
    /// Combine this action and the passed [`ActionSeed`].
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Hp(u8);
    ///
    /// #[derive(Event, Clone)]
    /// struct PlayerHit(Entity);
    ///
    /// Reactor::schedule(|task| async move{
    ///     task.will(Update, {
    ///         wait::event::read::<PlayerHit>()
    ///             .pipe(once::run(|In(PlayerHit(entity)): In<PlayerHit>, mut players: Query<&mut Hp>|{
    ///                 players.get_mut(entity).unwrap().0 -= 10;
    ///             }))
    ///     }).await;
    /// });
    /// ```
    fn pipe(self, seed: ActionSeed<O1, O2>) -> A;
}

impl<I1, O1, O2, A, ActionOrSeed> Pipe<I1, O1, O2, A> for ActionOrSeed
    where
        I1: 'static,
        O1: 'static,
        O2: 'static,
        ActionOrSeed: Remake<I1, O1, O2, A>
{
    #[inline(always)]
    fn pipe(self, seed: ActionSeed<O1, O2>) -> A {
        self.remake(|r1, o1, token, output| {
            PipeRunner::new(r1, o1, seed, token, output)
        })
    }
}

struct PipeRunner<O1, O2> {
    o1: Output<O1>,
    r1: BoxedRunner,
    r2: Option<BoxedRunner>,
    finished_r1: bool,
    seed: Option<ActionSeed<O1, O2>>,
    token: CancellationToken,
    output: Output<O2>,
}

impl<O1, O2> PipeRunner<O1, O2>
    where
        O1: 'static,
        O2: 'static
{
    pub fn new(
        r1: BoxedRunner,
        o1: Output<O1>,
        seed: ActionSeed<O1, O2>,
        token: CancellationToken,
        output: Output<O2>,
    ) -> PipeRunner<O1, O2> {
        Self {
            r1,
            r2: None,
            o1,
            token,
            output,
            finished_r1: false,
            seed: Some(seed),
        }
    }

    fn setup_second_runner(&mut self) {
        if let Some(o1) = self.o1.take() {
            self.finished_r1 = true;
            let Some(seed) = self.seed.take() else {
                self.o1.replace(o1);
                return;
            };
            let action = seed.with(o1);
            self.r2.replace(action.into_runner(self.token.clone(), self.output.clone()));
        }
    }
}

impl<O1, O2> Runner for PipeRunner<O1, O2>
    where
        O1: 'static,
        O2: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if !self.finished_r1 {
            self.r1.run(world);
        }
        self.setup_second_runner();
        if let Some(r2) = self.r2.as_mut() {
            r2.run(world);
        }
        self.output.is_some()
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, Update};
    use bevy_test_helper::event::DirectEvents;

    use crate::action::{delay, once};
    use crate::prelude::{Map, Reactor, Then, Through};
    use crate::tests::test_app;

    /// Make sure `Option::unwrap() on a None` does not occur.
    #[test]
    fn not_occur_unwrap_panic() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, delay::frames().with(2)
                    .map(|dummy| dummy)
                    .through(delay::frames().with(2))
                    .through(once::run(|| {}))
                    .then(once::event::app_exit()),
                ).await;
            }));
        });
        app.update();
        app.update();
        app.update();
        app.update();
        let mut er = ManualEventReader::<AppExit>::default();
        assert!(app.read_last_event(&mut er).is_some());
    }
}