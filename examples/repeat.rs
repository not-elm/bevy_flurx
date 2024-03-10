// use std::time::Duration;
// 
// use bevy::prelude::*;
// use store::selector::{delay, repeat};
// 
// use bevy_async_system::AsyncSystemPlugin;
// use bevy_async_system::scheduler::TaskScheduler;
// use bevy_async_system::task::once;
// 
// fn main() {
//     App::new()
//         .add_plugins((
//             MinimalPlugins,
//             AsyncSystemPlugin
//         ))
//         .add_systems(Startup, setup)
//         .run();
// }
// 
// 
// fn setup(mut scheduler: NonSendMut<TaskScheduler>) {
//     scheduler.schedule(|tc| async move {
//         tc.task(repeat::count(5, count_up)).await;
//         
//         tc.task(delay::time(Duration::from_secs(3))).await;
// 
//         println!("task canceled. Exit the application after 3 seconds.");
//         // Delay to make sure the system does not run.
//         tc.task(delay::time(Duration::from_secs(3))).await;
//         println!("App exit");
//         tc.task(once::app_exit()).await;
//     });
// }
// 
// 
// fn count_up(mut count: Local<u32>) {
//     *count += 1;
//     println!("count = {}", *count);
// }
// 

fn main() {
    
}
