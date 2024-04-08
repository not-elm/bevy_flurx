//! Provides the mechanism to pipe the actions. 

use bevy::prelude::World;

use crate::action::Action;
use crate::prelude::seed::ActionSeed;
use crate::runner::{BoxedActionRunner, CancellationToken, Output, Runner};

/// Provides the mechanism to pipe the actions. 
pub trait Pipe<I1, O1> {
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
    fn pipe<O2>(self, seed: ActionSeed<O1, O2>) -> Action<I1, O2>
        where
            O2: 'static;
}

impl<I1, O1, Act> Pipe<I1, O1> for Act
    where
        Act: Into<Action<I1, O1>> + 'static,
        I1: 'static,
        O1: Clone + 'static
{
    #[inline(always)]
    fn pipe<O2>(self, seed: ActionSeed<O1, O2>) -> Action<I1, O2>
        where
            O2: 'static
    {
        let Action(i1, s1) = self.into();
        ActionSeed::new(|input, token, output| {
            let o1 = Output::default();
            let r1 = s1.create_runner(input, token.clone(), o1.clone());
            PipeRunner::new(r1, o1, seed, token, output)
        })
            .with(i1)
    }
}

struct PipeRunner<O1, O2> {
    o1: Output<O1>,
    r1: BoxedActionRunner,
    r2: Option<BoxedActionRunner>,
    seed: Option<ActionSeed<O1, O2>>,
    token: CancellationToken,
    output: Output<O2>,
}

impl<O1, O2> PipeRunner<O1, O2>
    where
        O1: Clone + 'static,
        O2: 'static
{
    pub fn new(
        r1: BoxedActionRunner,
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
            seed: Some(seed),
        }
    }

    fn setup_second_runner(&mut self) {
        if let Some(o1) = self.o1.cloned() {
            let Some(action) = self.seed
                .take()
                .map(|p| p.with(o1))else {
                return;
            };
            self.r2.replace(action.into_runner(self.token.clone(), self.output.clone()));
        }
    }
}

impl<O1, O2> Runner for PipeRunner<O1, O2>
    where
        O1: Clone + 'static,
        O2: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if self.o1.is_none() {
            self.r1.run(world);
        }
        self.setup_second_runner();
        if let Some(r2) = self.r2.as_mut() {
            r2.run(world);
        }
        self.output.is_some()
    }
}
