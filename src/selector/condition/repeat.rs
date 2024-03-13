use bevy::prelude::{In, IntoSystem, Local, System};

// TODO: repeat
#[inline]
fn repeat<Sys, Input, Out>(count: usize, system: Sys) -> impl System<In=Input, Out=Option<Out>>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Out: 'static
{
    IntoSystem::into_system(system.pipe(move |In(out): In<Option<Out>>, mut count_now: Local<usize>| {
        let out = out?;

        if count <= *count_now {
            Some(out)
        } else {
            *count_now += 1;
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
    use crate::selector::condition::{once, wait, repeat::repeat};

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
        assert!(app.world.get_resource::<Test>().is_some());
    }
}
