use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::{Camera2dBundle, Color, Commands, In, NonSendMut, Query, TextBundle, Update};
use bevy::text::{Text, TextStyle};
use bevy::utils::default;
use reqwest::StatusCode;

use bevy_async_system::FlurxPlugin;
use bevy_async_system::scheduler::TaskScheduler;
use bevy_async_system::selector::condition::{once, with};

/// You can use [`reqwest`](reqwest).
///
/// I haven't confirmed any other async libraries yet, but I hope to be able to mix all async code together in the future.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, (
            setup_ui,
            setup_async_systems
        ))
        .run();
}


fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle::from_section("Loading", TextStyle {
        font_size: 80.,
        color: Color::BLACK,
        ..default()
    }));
}


fn setup_async_systems(mut scheduler: NonSendMut<TaskScheduler>) {
    scheduler.schedule(|task| async move {
        // This is my git repository uri.
        const URI: &str = "https://github.com/not-elm";
        let status_code = reqwest::get(URI).await.unwrap().status();
        task.will(Update, with(status_code, once::run(|In(status): In<StatusCode>, mut text: Query<&mut Text>| {
            text.single_mut().sections[0].value = status.to_string();
        }))).await;
    });
}