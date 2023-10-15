use bevy::app::{App, Startup, Update};
use bevy::asset::AssetServer;
use bevy::DefaultPlugins;
use bevy::prelude::{Camera2dBundle, Color, Commands, Query, Res, TextBundle};
use bevy::text::{Text, TextStyle};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::prelude::SpawnAsyncSystem;
use bevy_async_system::runner::once;


/// You can use [`reqwest`](reqwest).
///
/// I haven't confirmed any other async libraries yet, but I hope to be able to mix all async code together in the future.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, (
            setup_ui,
            setup_async_systems
        ))
        .run();
}


fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle::from_section("Loading", TextStyle {
        font_size: 80.,
        color: Color::BLACK,
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    }));
}


fn setup_async_systems(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        // This is my git repository uri.
        const URI: &str = "https://github.com/elm-register";
        let client = reqwest::get(URI).await;
        schedules.add_system(Update, once::run(move |mut text: Query<&mut Text>| {
            text.single_mut().sections[0].value = if let Ok(response) = client.as_ref() {
                format!("status code: {:?}", response.status())
            } else {
                "Failed".to_string()
            };
        })).await;
    });
}