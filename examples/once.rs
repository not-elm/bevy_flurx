use bevy::prelude::*;

use bevy_async_system::extension::ScheduleReactor;
use bevy_async_system::FlurxPlugin;
use bevy_async_system::selector::condition::once;

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


fn setup(world: &mut World) {
    world.schedule_reactor(|task| async move {
        task.will(First, once::run(println_system)).await;
        task.will(Update, once::state::set(ExampleState::Second)).await;
        task.will(Update, once::res::init::<Count>()).await;
        task.will(Update, once::non_send::init::<NonSendCount>()).await;

        let count = task.will(Update, once::run(return_count)).await;
        task.will(Update, once::res::insert(count)).await;
        task.will(Update, once::run(println_counts)).await;
        task.will(Update, once::event::app_exit()).await;
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