use std::time::Duration;

use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::{Camera2dBundle, Color, Commands, In, Query, TextBundle, Update, World};
use bevy::text::{Text, TextStyle};
use bevy::utils::default;
use reqwest::StatusCode;

use bevy_flurx::extension::ScheduleReactor;
use bevy_flurx::FlurxPlugin;
use bevy_flurx::selector::condition::{delay, once};

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

fn setup_async_systems(world: &mut World) {
    world.schedule_reactor(|task| async move {
        task.will(Update, delay::time(Duration::from_secs(1))).await;
        // This is my GitHub uri.
        const URI: &str = "https://github.com/not-elm";
        let status_code = reqwest::get(URI).await.unwrap().status();
        task.will(Update, once::run_with(status_code, |In(status): In<StatusCode>, mut text: Query<&mut Text>| {
            text.single_mut().sections[0].value = status.to_string();
        })).await;
        task.will(Update, delay::time(Duration::from_secs(1))).await;
        task.will(Update, once::event::app_exit()).await;
    });
}