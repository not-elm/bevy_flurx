use bevy::app::AppExit;
use bevy::prelude::{Commands, Event, EventWriter, In, IntoSystem, NextState, ResMut, Resource, States, World};
use flurx::selector::Selector;

use crate::selector::{run_system, WorldSelector};
use crate::world_ptr::WorldPtr;

struct Once<System, In, Out, Marker>(WorldSelector<System, In, Out, Marker>);


pub fn run<System, Out, Marker>(system: System) -> impl Selector<WorldPtr, Output=Out>
    where
        System: IntoSystem<(), Out, Marker> + Unpin + 'static,
        Out: 'static
{
    Once(WorldSelector::new((), system))
}

pub fn run_with<System, In, Out, Marker>(input: In, system: System) -> impl Selector<WorldPtr, Output=Out>
    where
        System: IntoSystem<In, Out, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    Once(WorldSelector::new(input, system))
}

pub fn send<E>(input: E) -> impl Selector<WorldPtr>
    where E: Event + Clone
{
    run_with(input, |input: In<E>, mut ew: EventWriter<E>| {
        ew.send(input.0);
    })
}

pub fn app_exit() -> impl Selector<WorldPtr> {
    send(AppExit)
}


pub fn set_state<S>(input: S) -> impl Selector<WorldPtr>
    where S: States + 'static
{
    run_with(input, |input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    })
}

pub fn init_non_send_resource<R>() -> impl Selector<WorldPtr>
    where R: Default + 'static
{
    run(|world: &mut World| {
        world.init_non_send_resource::<R>();
    })
}

pub fn insert_non_send_resource<R>(input: R) -> impl Selector<WorldPtr>
    where R: Clone + 'static
{
    run_with(input, |input: In<R>, world: &mut World| {
        world.insert_non_send_resource(input.0);
    })
}

pub fn init_resource<R>() -> impl Selector<WorldPtr>
    where R: Resource + Default + 'static
{
    run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}


pub fn insert_resource<R>(input: R) -> impl Selector<WorldPtr>
    where R: Resource + Clone + 'static
{
    run_with(input, |input: In<R>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}


impl<System, In, Out, Marker> Selector<WorldPtr> for Once<System, In, Out, Marker>
    where
        System: IntoSystem<In, Out, Marker> + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    type Output = Out;

    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        self.0.output(&world, Some(run_system(&self.0, &world)))
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, NonSendMut, Resource};

    use crate::AsyncSystemPlugin;
    use crate::scheduler::TaskScheduler;
    use crate::selector::once;

    #[test]
    fn once_run() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);
        #[derive(Resource)]
        struct Test;

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(once::run(|mut commands: Commands| {
                    commands.insert_resource(Test);
                })).await;
            })
        });

        app.update();
        assert!(app.world.get_resource::<Test>().is_some());
    }
}