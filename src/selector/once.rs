use bevy::app::AppExit;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{Commands, Event, EventWriter, In, IntoSystem, NextState, ResMut, Resource, States, World};

pub fn run<System, Input, Out, Marker>(system: System) -> BoxedSystem<Input, Option<Out>>
    where
        System: IntoSystem<Input, Out, Marker> + 'static,
        Input: Clone + 'static,
        Out: 'static
{
    Box::new(IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    })))
}

pub fn send<E>() -> BoxedSystem<E, Option<()>>
    where E: Event + Clone
{
    run(|input: In<E>, mut ew: EventWriter<E>| {
        ew.send(input.0);
    })
}

pub fn app_exit() -> BoxedSystem<AppExit, Option<()>> {
    send::<AppExit>()
}


pub fn set_state<S>() -> BoxedSystem<S, Option<()>>
    where S: States + 'static
{
    run(|input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    })
}

pub fn init_non_send_resource<R>() -> BoxedSystem<(), Option<()>>
    where R: Default + 'static
{
    run(|world: &mut World| {
        world.init_non_send_resource::<R>();
    })
}

pub fn insert_non_send_resource<R>() -> BoxedSystem<R, Option<()>>
    where R: Clone + 'static
{
    run(|In(resource): In<R>, world: &mut World| {
        world.insert_non_send_resource(resource);
    })
}

pub fn init_resource<R>() -> BoxedSystem<(), Option<()>>
    where R: Resource + Default + 'static
{
    run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}


pub fn insert_resource<R>() -> BoxedSystem<R, Option<()>>
    where R: Resource + Clone + 'static
{
    run(|input: In<R>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, First, Startup, Update};
    use bevy::prelude::{Commands, NonSendMut, Resource};

    use crate::FlurxPlugin;
    use crate::scheduler::TaskScheduler;
    use crate::selector::once;

    #[derive(Resource)]
    struct Test;

    #[test]
    fn once_run_on_pre_update() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |mut scheduler: NonSendMut<TaskScheduler>| {
                scheduler.schedule(|tc| async move {
                    tc.task(First, once::run(|mut commands: Commands| {
                        commands.insert_resource(Test);
                    })).await;
                })
            });
        app.update();
        app.update();
        app.update();
        assert!(app.world.get_resource::<Test>().is_some());
    }

    #[test]
    fn once_insert_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |mut scheduler: NonSendMut<TaskScheduler>| {
                scheduler.schedule(|task| async move {
                    task.task(Update, once::run(|mut commands: Commands| {
                        commands.insert_resource(Test);
                    })).await;
                })
            });
        app.update();
        assert!(app.world.get_resource::<Test>().is_some());
    }
}