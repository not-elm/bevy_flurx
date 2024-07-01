//! This example shows how to convert an asynchronous process such as HTTP communication into an action.

// TODO: Comment out when `bevy_egui` supports v0.14
fn main(){}

// use bevy::app::{App, Startup, Update};
// use bevy::DefaultPlugins;
// use bevy::prelude::{Camera2dBundle, Commands, default, Event, EventWriter, Local, Query, Res, Resource, Window, With};
// use bevy::window::PrimaryWindow;
// use bevy_egui::{egui, EguiContexts, EguiPlugin};
// use bevy_egui::egui::{Align2, Pos2};
// 
// use bevy_flurx::prelude::*;
// 
// 
// #[derive(Event, Clone)]
// struct RequestGet {
//     url: String,
// }
// 
// #[derive(Resource, Default)]
// struct ResponseInfo {
//     status: Option<reqwest::StatusCode>,
//     error_message: Option<String>,
// }
// 
// fn main() {
//     App::new()
//         .add_plugins((
//             DefaultPlugins,
//             EguiPlugin,
//             FlurxPlugin
//         ))
//         .init_resource::<ResponseInfo>()
//         .add_event::<RequestGet>()
//         .add_systems(Startup, (
//             spawn_camera,
//             spawn_reactor,
//         ))
//         .add_systems(Update, show_ui)
//         .run();
// }
// 
// fn spawn_camera(
//     mut commands: Commands
// ) {
//     commands.spawn(Camera2dBundle::default());
// }
// 
// fn spawn_reactor(
//     mut commands: Commands
// ) {
//     commands.spawn(Reactor::schedule(|task| async move {
//         loop {
//             task.will(Update, {
//                 wait::event::read::<RequestGet>()
//                     .pipe(effect::tokio::spawn(|RequestGet { url }| async move {
//                         match reqwest::get(url).await {
//                             Ok(response) => ResponseInfo {
//                                 status: Some(response.status()),
//                                 ..default()
//                             },
//                             Err(error) => ResponseInfo {
//                                 error_message: Some(error.to_string()),
//                                 ..default()
//                             }
//                         }
//                     }))
//                     .pipe(once::res::insert())
//             }).await;
//         }
//     }));
// }
// 
// fn show_ui(
//     mut cx: EguiContexts,
//     mut ew: EventWriter<RequestGet>,
//     mut input_text: Local<String>,
//     window: Query<&Window, With<PrimaryWindow>>,
//     response: Res<ResponseInfo>,
// ) {
//     egui::Window::new("bevy_flurx demo")
//         .pivot(Align2::CENTER_CENTER)
//         .default_pos(Pos2::new(window.single().width() / 2., window.single().height() / 2.))
//         .show(cx.ctx_mut(), |ui| {
//             ui.vertical(|ui| {
//                 ui.add(egui::TextEdit::singleline(&mut *input_text).hint_text("url"));
//                 if ui.button("submit").clicked() {
//                     ew.send(RequestGet {
//                         url: input_text.to_string()
//                     });
//                 }
//                 ui.allocate_space(egui::Vec2::new(1.0, 30.0));
//                 ui.separator();
//                 ui.vertical(|ui| {
//                     ui.label(format!("status code: {}", response.status.map(|s| s.to_string()).unwrap_or_default()));
//                     ui.label(format!("error message: {}", response.error_message.as_ref().map(|url| url.to_string()).unwrap_or_default()));
//                 });
//             });
//         });
// }
// 
// 
// 
