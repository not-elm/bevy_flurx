use bevy::prelude::World;

use crate::action::Action;
use crate::prelude::ActionSeed;
use crate::runner::{BoxedActionRunner, CancellationToken, Output, Runner};
use crate::runner::macros::output_combine;

/// Run until both tasks done.
///
/// ## Examples
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task|async move{
///     task.will(Update, wait::both(
///         wait::input::just_pressed().with(KeyCode::KeyR),
///         wait::event::read::<AppExit>()
///     )).await;
/// });
/// ```
pub fn both<LI, LO, RI, RO>(
    lhs: impl Into<Action<LI, LO>> + 'static,
    rhs: impl Into<Action<RI, RO>> + 'static,
) -> Action<(LI, RI), (LO, RO)>
    where
        RI: Clone + 'static,
        LI: Clone + 'static,
        LO: Send + 'static,
        RO: Send + 'static,
{
    let Action(i1, s1) = lhs.into();
    let Action(i2, s2) = rhs.into();
    ActionSeed::new(move |input: (LI, RI), token, output| {
        let o1 = Output::default();
        let o2 = Output::default();

        BothRunner {
            r1: s1.with(input.0).into_runner(token.clone(), o1.clone()),
            r2: s2.with(input.1).into_runner(token.clone(), o2.clone()),
            o1,
            o2,
            output,
            token,
        }
    })
        .with((i1, i2))
}

struct BothRunner<O1, O2> {
    r1: BoxedActionRunner,
    r2: BoxedActionRunner,
    o1: Output<O1>,
    o2: Output<O2>,
    output: Output<(O1, O2)>,
    token: CancellationToken,
}

impl<O1, O2> Runner for BothRunner<O1, O2>
    where
        O1: 'static,
        O2: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if self.o1.is_none() {
            self.r1.run(world);
        }
        if self.o2.is_none() {
            self.r2.run(world);
        }
        output_combine!(&self.o1, &self.o2, self.output)
    }
}