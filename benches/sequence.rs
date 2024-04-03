//! Testing the difference between not using Flurx and using Flurx in a simple countdown.

use bevy::app::{App, Startup};
use bevy::prelude::{ResMut, Resource, Update, World};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::{FlurxPlugin, sequence};
use bevy_flurx::extension::ScheduleReactor;
use bevy_flurx::prelude::once;

#[derive(Resource, Default)]
struct Exit(bool);

fn default_version(c: &mut Criterion) {
    c.bench_function("default", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(FlurxPlugin)
                .init_resource::<Exit>()
                .add_systems(Startup, |world: &mut World| {
                    world.schedule_reactor(|task| async move {
                        task.will(Update, once::run(|| {})).await;
                        task.will(Update, once::run(|mut exit: ResMut<Exit>| {
                            exit.0 = true;
                        })).await;
                    });
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
                .add_systems(Startup, |world: &mut World| {
                    world.schedule_reactor(|task| async move {
                        task.will(Update, sequence! {
                            once::run(|| {}),
                            once::run(|mut exit: ResMut<Exit>| {
                                exit.0 = true;
                            })
                         }).await;
                    });
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

criterion_group!(benches, default_version, flurx_version);
criterion_main!(benches);