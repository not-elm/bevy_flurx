use bevy::prelude::{In, System, World};

use crate::selector::condition::once;

#[inline]
pub fn init<R>() -> impl System<In=(), Out=Option<()>>
    where R: Default + 'static
{
    once::run(|world: &mut World| {
        world.init_non_send_resource::<R>();
    })
}

#[inline]
pub fn insert<R>() -> impl System<In=R, Out=Option<()>>
    where R: Clone + 'static
{
    once::run(|In(resource): In<R>, world: &mut World| {
        world.insert_non_send_resource(resource);
    })
}

#[inline]
pub fn remove<R>() -> impl System<In=(), Out=Option<()>>
    where R: 'static
{
    once::run(|world: &mut World| {
        world.remove_non_send_resource::<R>();
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::once::non_send;
    use crate::selector::condition::with;
    use crate::tests::TestResource;

    #[test]
    fn init_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::init::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_some());
    }

    #[test]
    fn insert_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, with(TestResource, non_send::insert())).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_some());
    }


    #[test]
    fn remove_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_resource::<TestResource>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::remove::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_none());
    }

    #[test]
    fn success_run_all_schedule_labels() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, with(AppExit, non_send::insert())).await;
                    println!("First finished");
                    task.will(First, with(AppExit, non_send::insert())).await;
                    println!("PreUpdate finished");
                    task.will(First, with(AppExit, non_send::insert())).await;
                    println!("Update finished");
                    task.will(First, with(AppExit, non_send::insert())).await;
                    println!("PostUpdate finished");
                    task.will(First, with(AppExit, non_send::insert())).await;
                    println!("Last finished");
                });
            });

        println!("First");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("PreUpdate");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("Update");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("PostUpdate");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("Last");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("After Reactor Finished");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_none());
    }
}