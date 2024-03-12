use std::cell::RefMut;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{Event, EventReader, In, IntoSystem};

pub fn until<Input, System, Marker>(system: System) -> BoxedSystem<Input, Option<()>>
    where
        System: IntoSystem<Input, bool, Marker> + 'static
{
    Box::new(IntoSystem::into_system(
        system
            .pipe(|In(finish): In<bool>| {
                if finish {
                    Some(())
                } else {
                    None
                }
            })
    ))
}

pub fn event<E>() -> BoxedSystem<(), Option<E>>
    where E: Event + Clone
{
    Box::new(IntoSystem::into_system(
        |mut er: EventReader<E>| {
            er.read().next().cloned()
        }
    ))
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{In, Local, NonSendMut, Update};

    use crate::FlurxPlugin;
    use crate::scheduler::TaskScheduler;
    use crate::selector::{once, wait};

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task(Update, wait::until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 2
                })).await;

                task.task_with(Update, AppExit, once::insert_non_send_resource()).await;
            })
        });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }


    #[test]
    fn count_up_until_with_input() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|mut scheduler: NonSendMut<TaskScheduler>| {
            scheduler.schedule(|task| async move {
                task.task_with(Update, 1, wait::until(|input: In<u32>, mut count: Local<u32>| {
                    *count += 1 + input.0;
                    *count == 4
                })).await;

                task.task_with(Update, AppExit, once::insert_non_send_resource()).await;
            })
        });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn wait_event() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |mut scheduler: NonSendMut<TaskScheduler>|{
                
            });
    }
}