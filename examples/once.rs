use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_async_system::prelude::*;

fn main() {
    App::new()
        .add_state::<ExampleState>()
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


fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, once::run(println_system)).await;
        schedules.add_system(Update, once::set_state(ExampleState::Second)).await;
        schedules.add_system(Update, once::init_resource::<Count>()).await;
        schedules.add_system(Update, once::init_non_send_resource::<NonSendCount>()).await;

        let count = schedules.add_system(Update, once::run(return_count)).await;
        schedules.add_system(Update, once::insert_resource(count)).await;
        schedules.add_system(Update, once::run(println_counts)).await;

        schedules.add_system(Update, once::send(AppExit)).await;
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