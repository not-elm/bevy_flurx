use bevy::prelude::*;

use bevy_async_system::FlurxPlugin;
use bevy_async_system::scheduler::TaskScheduler;
use bevy_async_system::selector::condition::{once, with};

fn main() {
    App::new()
        .init_state::<ExampleState>()
        .add_plugins((
            MinimalPlugins,
            FlurxPlugin
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
        tc.will(First, once::run(println_system)).await;
        tc.will(Update, with(ExampleState::Second, once::state::set())).await;
        tc.will(Update, once::res::init::<Count>()).await;
        tc.will(Update, once::non_send::init::<NonSendCount>()).await;

        let count = tc.will(Update, once::run(return_count)).await;
        tc.will(Update, with(count, once::res::insert())).await;
        tc.will(Update, once::run(println_counts)).await;
        tc.will(Update, once::event::app_exit()).await;
    });
}


fn println_system() {
    println!("hello!");
}


fn return_count() -> Count {
    Count(30)
}

fn println_counts(
    count: Res<Count>,
    non_send_count: NonSend<NonSendCount>,
) {
    println!("{count:?}");
    println!("{non_send_count:?}");
}