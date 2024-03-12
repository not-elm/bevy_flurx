use bevy::prelude::IntoSystem;
use flurx::selector::Selector;

use crate::world_ptr::WorldPtr;
use crate::selector::{run_system, WorldSelector};

struct Wait<System, In, Marker> {
    inner: WorldSelector<System, In, bool, Marker>,
    is_while: bool,
}


pub fn while_<System, Marker>(system: System) -> impl Selector<WorldPtr>
    where
        System: IntoSystem<(), bool, Marker> + Unpin + 'static
{
    while_with((), system)
}


pub fn while_with<System, In, Marker>(input: In, system: System) -> impl Selector<WorldPtr>
    where
        System: IntoSystem<In, bool, Marker> + Unpin + 'static,
        In: Clone + 'static
{
    Wait {
        inner: WorldSelector::new(input, system),
        is_while: false,
    }
}

pub fn until<System, Marker>(system: System) -> impl Selector<WorldPtr>
    where
        System: IntoSystem<(), bool, Marker> + Unpin + 'static
{
    until_with((), system)
}


pub fn until_with<System, In, Marker>(input: In, system: System) -> impl Selector<WorldPtr>
    where
        System: IntoSystem<In, bool, Marker> + Unpin + 'static,
        In: Clone + 'static
{
    Wait {
        inner: WorldSelector::new(input, system),
        is_while: false,
    }
}


impl<System, In, Marker> Selector<WorldPtr> for Wait<System, In, Marker>
    where
        System: IntoSystem<In, bool, Marker> + Unpin + 'static,
        In: Clone + 'static
{
    type Output = ();

    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        self.inner.output(&world, match (self.is_while, run_system(&self.inner, &world)) {
            // while
            (true, false) => Some(()),
            // until
            (false, true) => Some(()),
            _ => None
        })
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{In, Local, NonSendMut};

    use crate::AsyncSystemPlugin;
    use crate::scheduler::TaskScheduler;
    use crate::selector::{once, wait};

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(wait::until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 2
                })).await;

                task.task(once::insert_non_send_resource(AppExit)).await;
            })
        });

        app.update();
        app.update();

        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }


     #[test]
    fn count_up_until_with() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(wait::until_with(1, |input: In<u32>, mut count: Local<u32>| {
                    *count += 1 + input.0;
                    *count == 4
                })).await;

                task.task(once::insert_non_send_resource(AppExit)).await;
            })
        });

        app.update();
        app.update();

        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn count_up_while_less_than_2() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(wait::while_(|mut count: Local<u32>| {
                    *count += 1;
                    *count < 2
                })).await;

                task.task(once::insert_non_send_resource(AppExit)).await;
            })
        });

        app.update();
        app.update();

        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }


    #[test]
    fn count_up_while_with() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(wait::while_with(1, |input: In<u32>, mut count: Local<u32>| {
                    *count += 1 + input.0;
                    *count != 4
                })).await;

                task.task(once::insert_non_send_resource(AppExit)).await;
            })
        });

        app.update();
        app.update();

        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}