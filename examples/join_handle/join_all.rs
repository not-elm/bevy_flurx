use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use futures::future::join_all;
use futures::FutureExt;

use bevy_async_system::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup_async_systems)
        .run();
}


fn setup_async_systems(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, once::run(setup_blocks)).await;
        schedules.add_system(PreUpdate, once::insert_resource(MoveTimer(Timer::from_seconds(5., TimerMode::Once)))).await;
        schedules.add_system(PreUpdate, once::init_resource::<RotateTimer>()).await;

        let h1 = schedules.add_system(FixedUpdate, wait::until(move_block1));
        let h2 = schedules.add_system(FixedUpdate, wait::until(rotate_block1));
        let h3 = schedules.add_system(FixedUpdate, wait::until(rotate_block2));
        join_all(vec![h1.boxed(), h2.boxed(), h3.boxed()]).await;
        schedules.add_system(PreUpdate, once::run(setup_count_down_ui)).await;
        schedules.add_system(PreUpdate, once::init_resource::<CountDownTimer>()).await;
        schedules.add_system(Update, wait::until(count_down)).await;

        info!("AppExit");
        schedules.add_system(Update, once::send(AppExit)).await;
    });
}


#[derive(Resource, Clone)]
struct MoveTimer(Timer);

#[derive(Resource, Clone)]
struct RotateTimer(Timer);

impl Default for RotateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3., TimerMode::Once))
    }
}

#[derive(Component)]
struct Block1;


#[derive(Component)]
struct Block2;

fn setup_blocks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });

    commands.spawn((
        Block1,
        Name::new("Block1"),
        PbrBundle {
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                ..default()
            }),
            mesh: meshes.add(shape::Box::default().into()),
            transform: Transform::from_xyz(3., -3., 3.),
            ..default()
        }
    ));


    commands.spawn((
        Block2,
        Name::new("Block2"),
        PbrBundle {
            material: materials.add(StandardMaterial {
                base_color: Color::GREEN,
                ..default()
            }),
            mesh: meshes.add(shape::Box::default().into()),
            ..default()
        }
    ));
}


fn move_block1(
    mut block1: Query<&mut Transform, With<Block1>>,
    time: Res<Time>,
) -> bool {
    let mut transform = block1.single_mut();
    transform.translation.y += time.delta_seconds();

    2. <= transform.translation.y
}

fn rotate_block1(
    mut timer: ResMut<MoveTimer>,
    mut block1: Query<&mut Transform, With<Block1>>,
    time: Res<Time>,
) -> bool {
    block1.single_mut().rotate(Quat::from_rotation_y(time.delta_seconds().sin()));

    timer.0.tick(time.delta()).just_finished()
}


fn rotate_block2(
    mut timer: ResMut<RotateTimer>,
    mut block2: Query<&mut Transform, With<Block2>>,
    time: Res<Time>,
) -> bool {
    block2.single_mut().rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds().sin()));

    timer.0.tick(time.delta()).just_finished()
}


#[derive(Component)]
struct CountDownText;

fn setup_count_down_ui(
    mut commands: Commands,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.5)),
        ..default()
    })
        .with_children(|parent| {
            parent.spawn((
                CountDownText,
                TextBundle::from_sections(vec![
                    TextSection::new("App will close in ", TextStyle { font_size: 50., ..default() }),
                    TextSection::new("3", TextStyle { font_size: 70., color: Color::PURPLE, ..default() }),
                    TextSection::new(" seconds.", TextStyle { font_size: 50., ..default() }),
                ])
            ));
        });
}


#[derive(Resource)]
struct CountDownTimer(Timer);

impl Default for CountDownTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}


fn count_down(
    mut count_down: Query<&mut Text, With<CountDownText>>,
    mut timer: ResMut<CountDownTimer>,
    mut count: Local<u8>,
    time: Res<Time>,
) -> bool {
    if timer.0.tick(time.delta()).just_finished() {
        *count += 1;
        count_down.single_mut().sections[1].value = (3 - *count).to_string();
    }

    *count == 3
}