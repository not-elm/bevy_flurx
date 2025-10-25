//! This example introduces [`inspect`].
//!
//! [`inspect`] creates an [`ActionSeed`] that clones its input, passing one clone to the provided seed for further processing while forwarding the original input as the output.
//! This is useful for observing or inspecting input values by performing side-effects (like logging or metrics) without altering the main input-output chain.
//!
//! [`inspect`]:inspect::inspect

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default(), FlurxPlugin))
        .insert_resource(PlayerHp(100))
        .add_message::<Damage>()
        .add_systems(Startup, spawn_reactor)
        .add_systems(Update, hit.run_if(on_timer(Duration::from_secs(1))))
        .run();
}

#[derive(Resource)]
struct PlayerHp(isize);

#[derive(Message, Clone)]
struct Damage(usize);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        let action = wait::output(||Some(1));
        // Action -> Future<Output=i32>
        let f = task.will(Update, action).await;


        task.will(Update, {
            wait::event::read::<Damage>()
                .inspect(once::run(|In(damage): In<Damage>| {
                    info!("Hit damage: {}", damage.0);
                }))
                .pipe(once::run(
                    |In(damage): In<Damage>, mut player_hp: ResMut<PlayerHp>| {
                        player_hp.0 -= damage.0 as isize;
                        info!("Player HP: {}", player_hp.0);
                    },
                ))
                .then(once::event::app_exit_success())
        })
        .await;
    }));
}

fn hit(mut ew: MessageWriter<Damage>) {
    ew.write(Damage(10));
}
