// use bevy::prelude::World;
// 
// use crate::action::Action;
// use crate::action::seed::ActionSeed;
// use crate::runner::{BoxedActionRunner, CancellationToken, Output, Runner};
// 
// pub fn repeat<I, O>(count: usize, seed: ActionSeed<I, O>) -> ActionSeed<I, O>
//     where
//         I: Clone + 'static,
//         O: 'static,
// {
//     ActionSeed::new(move |input, token, output| {
//         RepeatRunner {
//             action: seed.with(input),
//             repeat: count,
//             runner: None,
//             count: 0,
//             output,
//             token,
//             tmp_output: Output::default(),
//         }
//     })
// }
// 
// struct RepeatRunner<I, O> {
//     action: Action<I, O>,
//     runner: Option<BoxedActionRunner>,
//     count: usize,
//     repeat: usize,
//     token: CancellationToken,
//     output: Output<O>,
//     tmp_output: Output<O>,
// }
// 
// impl<I, O> Runner for RepeatRunner<I, O>
//     where 
//         I: Clone + 'static
// {
//     fn run(&mut self, world: &mut World) -> bool {
//         if self.runner.is_none() {
//             self.runner.replace(self.action.clone().to_runner(self.token.clone(), self.tmp_output.clone()));
//         }
//         self.runner.as_mut().unwrap().run(world);
//         if let Some(output) = self.tmp_output.take() {
//             if self.repeat <= self.count {
//                 self.output.replace(output);
//                 true
//             } else {
//                 self.runner = None;
//                 self.count += 1;
//                 false
//             }
//         } else {
//             false
//         }
//     }
// }
// 
// #[cfg(test)]
// mod tests {
//     use bevy::app::{AppExit, First, Startup};
//     use bevy::ecs::event::ManualEventReader;
//     use bevy::prelude::{Commands, Local, Resource};
//     use bevy_test_helper::event::DirectEvents;
// 
//     use crate::action::{once, wait};
//     use crate::action::repeat::repeat;
//     use crate::action::sequence::Then;
//     use crate::prelude::Reactor;
//     use crate::tests::test_app;
// 
//     #[derive(Eq, PartialEq, Debug, Default, Resource)]
//     struct Count(usize);
// 
//     #[test]
//     fn repeat_local_system() {
//         let mut app = test_app();
//         app.add_systems(Startup, |mut commands: Commands| {
//             commands.spawn(Reactor::schedule(|task| async move {
//                 task.will(First, repeat(1, wait::until(|mut local: Local<usize>| {
//                     *local += 1;
//                     *local == 2
//                 }))
//                     .then(once::event::app_exit()),
//                 ).await;
//             }));
//         });
//         let mut er = ManualEventReader::<AppExit>::default();
//         app.update();
//         assert!(app.read_last_event(&mut er).is_none());
//         app.update();
//         assert!(app.read_last_event(&mut er).is_none());
// 
//         app.update();
//         assert!(app.read_last_event(&mut er).is_none());
//         app.update();
//         assert!(app.read_last_event(&mut er).is_some());
//     }
// }