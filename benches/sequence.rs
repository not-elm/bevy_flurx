//! Testing the difference between not using sequence! and using.
#![allow(missing_docs)]

use bevy::app::{App, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, ResMut, Resource, Update};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::{FlurxPlugin, sequence};
use bevy_flurx::prelude::{Reactor, once};

#[derive(Resource, Default)]
struct Exit(bool);

fn without_sequence(c: &mut Criterion) {
    c.bench_function("without_sequence", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, once::run(|| {})).await;
                        task.will(Update, once::run(|mut exit: ResMut<Exit>| {
                            exit.0 = true;
                        })).await;
                    }));
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

fn with_sequence(c: &mut Criterion) {
    c.bench_function("with_sequence", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, sequence! {
                            once::run(|| {}),
                            once::run(|mut exit: ResMut<Exit>| {
                                exit.0 = true;
                            })
                         }).await;
                    }));
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

criterion_group!(sequence, without_sequence, with_sequence);
criterion_main!(sequence);