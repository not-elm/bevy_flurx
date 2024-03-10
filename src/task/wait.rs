use bevy::prelude::IntoSystem;
use store::selector::StateSelector;

use crate::store::WorldPointer;
use crate::task::selector::{run_system, WorldSelector};

struct Wait<System, In, Marker> {
    inner: WorldSelector<System, In, bool, Marker>,
    is_while: bool,
}


pub fn while_<System, Marker>(system: System) -> impl StateSelector<WorldPointer>
    where
        System: IntoSystem<(), bool, Marker> + Clone + Unpin + 'static
{
    Wait {
        inner: WorldSelector::new((), system),
        is_while: false,
    }
}

pub fn until<System, Marker>(system: System) -> impl StateSelector<WorldPointer>
    where
        System: IntoSystem<(), bool, Marker> + Clone + Unpin + 'static
{
    Wait {
        inner: WorldSelector::new((), system),
        is_while: false,
    }
}


impl<System, In, Marker> StateSelector<WorldPointer> for Wait<System, In, Marker>
    where
        System: IntoSystem<In, bool, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static
{
    type Output = ();

    fn select(&self, state: &WorldPointer) -> Option<Self::Output> {
        match (self.is_while, run_system(&self.inner, state)) {
            // while
            (true, false) => Some(()),
            // until
            (false, true) => Some(()),
            _ => None
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Local, NonSendMut};

    use crate::AsyncSystemPlugin;
    use crate::scheduler::BevyScheduler;
    use crate::task::{once, wait};

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(AsyncSystemPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<BevyScheduler>| {
            scheduler.schedule(|task| async move {
                task.run(wait::until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 2
                })).await;

                task.run(once::insert_non_send_resource(AppExit)).await;
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

        app.world.run_system_once(|mut scheduler: NonSendMut<BevyScheduler>| {
            scheduler.schedule(|task| async move {
                task.run(wait::while_(|mut count: Local<u32>| {
                    *count += 1;
                    *count < 2
                })).await;

                task.run(once::insert_non_send_resource(AppExit)).await;
            })
        });

        app.update();
        app.update();

        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}