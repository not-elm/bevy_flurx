use std::hash::Hasher;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::{In, IntoSystem, Local, World};

use crate::selector::condition::{ReactorSystemConfigs, with, WithInput};

// TODO: repeat
#[inline]
pub fn repeat<Sys, Input, Out, Marker>(count: usize, system: Sys) -> impl ReactorSystemConfigs<WithInput, In=Input, Out=Out>
    where
        Input: Clone + 'static,
        Out: 'static,
        Sys: ReactorSystemConfigs<Marker, In=Input, Out=Out>,
{
    use bevy::prelude::System;
    let (input, mut system) = system.into_configs();
    let count_now = AtomicUsize::new(0);

    with(input, IntoSystem::into_system(move |In(input): In<Input>,
                                              world: &mut World,
                                              mut init: Local<bool>| {
        if !*init {
            system.initialize(world);
            *init = true;
        }

        let output = unsafe { system.run_unsafe(input, world.as_unsafe_world_cell()) }?;
        system.apply_deferred(world);
        if count <= count_now.fetch_add(1, Ordering::Relaxed) {
            Some(output)
        } else {
            println!("{:?}", system.component_access());
            world.get_resource_by_id().unwrap().
            system.initialize(world);
            None
        }
    }))
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, First, Startup};
    use bevy::prelude::{Local, ResMut, Resource, World};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{once, repeat::repeat, wait};

    #[derive(Resource, Clone, Default)]
    struct Test(usize);

    #[test]
    fn repeat_3_times() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_resource::<Test>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, repeat(2, once::run(|mut test: ResMut<Test>| {
                        test.0 += 1;
                    }))).await;
                });
            });
        app.update();
        assert_eq!(app.world.resource::<Test>().0, 1);
        app.update();
        assert_eq!(app.world.resource::<Test>().0, 2);
        app.update();
        assert_eq!(app.world.resource::<Test>().0, 3);
        app.update();
        assert_eq!(app.world.resource::<Test>().0, 3);
    }


    // FIXME: このテストは失敗します。
    #[test]
    fn when_repeat_reset_local() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, repeat(2, wait::until(|mut count: Local<usize>| {
                        *count += 1;
                        println!("{count:?}");
                        *count == 2
                    }))).await;
                    task.will(First, once::res::init::<Test>()).await;
                });
            });
        app.update();
        assert!(app.world.get_resource::<Test>().is_none());
        app.update();
        assert!(app.world.get_resource::<Test>().is_none());
        app.update();
        assert!(app.world.get_resource::<Test>().is_none());
        app.update();
        assert!(app.world.get_resource::<Test>().is_none());
        app.update();
        app.update();
        app.update();
        assert!(app.world.get_resource::<Test>().is_some());
    }
}
