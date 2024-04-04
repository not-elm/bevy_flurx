//! [`repeat`] is still under development.

// use std::sync::atomic::{AtomicUsize, Ordering};
// 
// use bevy::prelude::{In, IntoSystem, World};
// 
// use crate::selector::condition::{ReactorSystemConfigs, with, WithInput};
// 
// // TODO: repeat
// #[inline]
// fn repeat<Sys, Input, Out, Marker>(count: usize, system: Sys) -> impl ReactorSystemConfigs<WithInput, In=Input, Out=Out>
//     where
//         Input: Clone + 'static,
//         Out: 'static,
//         Sys: ReactorSystemConfigs<Marker, In=Input, Out=Out>,
// {
//     let (input, system) = system.into_configs();
//     let mut system = Some(system);
//     let count_now = AtomicUsize::new(0);
//     let mut system_id = Option::None;
//     with(input, IntoSystem::into_system(move |In(input): In<Input>, world: &mut World| {
//         if let Some(system) = system.take() {
//             system_id.replace(world.register_system(system));
//         }
//         let Some(id) = system_id.as_ref() else {
//             panic!("unreachable");
//         };
//         let output = world.run_system_with_input(*id, input).unwrap()?;
//         {
//              world.syste(*id).unwrap();
//         }
// 
//         if count <= count_now.fetch_add(1, Ordering::Relaxed) {
//             Some(output)
//         } else {
//             // system.initialize(world);
//             // system_id.replace(world.register_boxed_system(system));
//             None
//         }
//     }))
// }
// 
// 
// #[cfg(test)]
// mod tests {
//     use bevy::app::{App, First, Startup};
//     use bevy::prelude::{Local, ResMut, Resource, World};
// 
//     use crate::extension::ScheduleReactor;
//     use crate::FlurxPlugin;
//     use crate::selector::condition::{once, repeat::repeat, wait};
// 
//     #[derive(Resource, Clone, Default)]
//     struct Test(usize);
// 
//     #[test]
//     fn repeat_3_times() {
//         let mut app = App::new();
//         app
//             .add_plugins(FlurxPlugin)
//             .init_resource::<Test>()
//             .add_systems(Startup, |world: &mut World| {
//                 world.schedule_reactor(|task| async move {
//                     task.will(First, repeat(2, once::run(|mut test: ResMut<Test>| {
//                         test.0 += 1;
//                     }))).await;
//                 });
//             });
//         app.update();
//         assert_eq!(app.world.resource::<Test>().0, 1);
//         app.update();
//         assert_eq!(app.world.resource::<Test>().0, 2);
//         app.update();
//         assert_eq!(app.world.resource::<Test>().0, 3);
//         app.update();
//         assert_eq!(app.world.resource::<Test>().0, 3);
//     }
// 
//     #[test]
//     fn when_repeat_reset_local() {
//         let mut app = App::new();
//         app
//             .add_plugins(FlurxPlugin)
//             .add_systems(Startup, |world: &mut World| {
//                 world.schedule_reactor(|task| async move {
//                     task.will(First, repeat(1, wait::until(|mut count: Local<usize>| {
//                         *count += 1;
//                         println!("{count:?}");
//                         *count == 2
//                     }))).await;
//                     task.will(First, once::res::init::<Test>()).await;
//                 });
//             });
//         app.update();
//         assert!(app.world.get_resource::<Test>().is_none());
//         app.update();
//         assert!(app.world.get_resource::<Test>().is_none());
// 
//         app.update();
//         assert!(app.world.get_resource::<Test>().is_none());
//         app.update();
//         assert!(app.world.get_resource::<Test>().is_none());
// 
//         app.update();
//         assert!(app.world.get_resource::<Test>().is_some());
//     }
// }
