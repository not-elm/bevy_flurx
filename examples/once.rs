use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_async_system::AsyncSystemPlugin;

use bevy_async_system::scheduler::TaskScheduler;
use bevy_async_system::selector::once;

fn main() {
    App::new()
        .init_state::<ExampleState>()
        .add_plugins((
            MinimalPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}


#[derive(Eq, PartialEq, Copy, Clone, Debug, Default, States, Hash)]
enum ExampleState {
    #[default]
    First,
    Second,
}

#[derive(Resource, Eq, PartialEq, Default, Clone, Debug)]
struct Count(usize);


#[derive(Eq, PartialEq, Default, Clone, Debug)]
struct NonSendCount(usize);


fn setup(mut scheduler: NonSendMut<TaskScheduler>) {
    scheduler.schedule(|tc| async move {
        tc.task(once::run(println_system)).await;
        tc.task(once::set_state(ExampleState::Second)).await;
        tc.task(once::init_resource::<Count>()).await;
        tc.task(once::init_non_send_resource::<NonSendCount>()).await;

        let count = tc.task(once::run(return_count)).await;
        tc.task(once::insert_resource(count)).await;
        tc.task(once::run(println_counts)).await;

        tc.task(once::send(AppExit)).await;
    });
}


fn println_system() {
    println!("hello!");
}


fn return_count() -> Count{
    Count(30)
}

fn println_counts(
    count: Res<Count>,
    non_send_count: NonSend<NonSendCount>
){
    println!("{count:?}");
    println!("{non_send_count:?}");
}