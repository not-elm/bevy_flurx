use crate::action::seed::ActionSeed;
use crate::action::TaskAction;
use crate::private::RunnerIntoAction;
use crate::runner::pipe::PipeRunner;

pub trait Pipe<I1, O1> {
    fn pipe<O2>(self, action: impl ActionSeed<O1, O2> + 'static) -> impl TaskAction<I1, O2>
        where
            O2: 'static;
}

impl<I1, O1, Action> Pipe<I1, O1> for Action
    where
        Action: TaskAction<I1, O1> + 'static,
        I1: 'static,
        O1: Clone + 'static
{
    #[inline(always)]
    fn pipe<O2>(self, action: impl ActionSeed<O1, O2> + 'static) -> impl TaskAction<I1, O2>
        where
            O2: 'static
    {
        RunnerIntoAction::new(PipeRunner::new(self, action))
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, In, Resource, Update};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{once, wait};
    use crate::prelude::{Flurx, Pipe, Then};
    use crate::tests::test_app;

    #[derive(Resource, Debug, Eq, PartialEq)]
    struct Num(usize);

    #[test]
    fn one_pipe() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                task.will(Update, {
                    once::run(|| { 1 + 1 })
                        .pipe(once::run(|In(input): In<usize>, mut cmd: Commands| {
                            cmd.insert_resource(Num(input));
                        }))
                })
                    .await;
            }));
        });
        app.update();
        app.assert_resource_eq(Num(2));
    }

    #[test]
    fn pipe_3() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                task.will(Update, once::run(|| { 1 + 1 })
                    .pipe(once::run(|In(num): In<usize>| { num * num }))
                    .pipe(wait::until(|In(num): In<usize>| {
                        num == 4
                    }))
                    .then(once::event::app_exit()),
                ).await;
            }));
        });
        let mut er = ManualEventReader::<AppExit>::default();
        app.update();
        assert!(app.read_last_event(&mut er).is_some());
    }
}