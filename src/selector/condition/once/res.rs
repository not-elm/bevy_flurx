use bevy::prelude::{Commands, In, Resource, System};

use crate::selector::condition::once;

#[inline]
pub fn init<R>() -> impl System<In=(), Out=Option<()>>
    where R: Resource + Default + 'static
{
    once::run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}

#[inline]
pub fn insert<R>() -> impl System<In=R, Out=Option<()>>
    where R: Resource + 'static
{
    once::run(|input: In<R>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}

#[inline]
pub fn remove<R>() -> impl System<In=(), Out=Option<()>>
    where R: Resource + 'static
{
    once::run(|mut commands: Commands| {
        commands.remove_resource::<R>();
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::once::res;
    use crate::selector::condition::with;
    use crate::tests::TestResource;

    #[test]
    fn init_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, res::init::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_some());
    }

    #[test]
    fn insert_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, with(TestResource, res::insert())).await;
                });
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_some());
    }

    #[test]
    fn remove_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_resource::<TestResource>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, res::remove::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_none());
    }
}