use bevy::prelude::World;

use crate::action::remake::Remake;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

pub trait Map<I1, O1, O2, ActionOrSeed> {
    fn map(self, f: impl FnOnce(O1) -> O2 + 'static) -> ActionOrSeed;
}

impl<I, O, O2, A, Re> Map<I, O, O2, A> for Re
    where
        I: 'static,
        O: 'static,
        O2: 'static,
        Re: Remake<I, O, O2, A> + 'static
{
    #[inline]
    fn map(self, f: impl FnOnce(O) -> O2 + 'static) -> A {
        self.remake(|r1, o1, token, output| {
            MapRunner {
                r1,
                o1,
                token,
                output,
                map: Some(f),
            }
        })
    }
}

struct MapRunner<O1, O2, F> {
    token: CancellationToken,
    r1: BoxedRunner,
    o1: Output<O1>,
    output: Output<O2>,
    map: Option<F>,
}

impl<O1, O2, F> Runner for MapRunner<O1, O2, F>
    where F: FnOnce(O1) -> O2 + 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }
        self.r1.run(world);
        if let Some(o) = self.o1.take() {
            self.output.replace(self.map.take().unwrap()(o));
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::Commands;

    use crate::action::once;
    use crate::prelude::{Map, Reactor};
    use crate::tests::test_app;

    #[test]
    fn map_num_to_string() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|| 3).map(|num| format!("{num}"))).await;
            }));
        });
    }
}