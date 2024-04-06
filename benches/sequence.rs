//! Testing the difference between not using sequence! and using.
#![allow(missing_docs)]

use bevy::app::{App, Startup};
use bevy::prelude::{Commands, ResMut, Resource, Update};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::{FlurxPlugin, sequence};
use bevy_flurx::prelude::{Flurx, once};

#[derive(Resource, Default)]
struct Exit(bool);

fn default_version(c: &mut Criterion) {
    c.bench_function("default", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(FlurxPlugin)
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Flurx::schedule(|task| async move {
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

fn flurx_version(c: &mut Criterion) {
    c.bench_function("sequence", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(FlurxPlugin)
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Flurx::schedule(|task| async move {
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

criterion_group!(benches, default_version, flurx_version);
criterion_main!(benches);