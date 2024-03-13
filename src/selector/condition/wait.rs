use bevy::prelude::{Event, EventReader, In, IntoSystem, System};

use crate::selector::condition::{ReactorSystemConfigs, wait, with};

pub mod state;

#[inline]
pub fn output<Sys, Input, Out, Marker>(system: Sys) -> impl System<In=Input, Out=Option<Out>>
    where
        Sys: IntoSystem<Input, Option<Out>, Marker>,
        Input: 'static,
        Out: 'static
{
    IntoSystem::into_system(system)
}

#[inline]
pub fn until<Input, Sys, Marker>(system: Sys) -> impl System<In=Input, Out=Option<()>>
    where
        Sys: IntoSystem<Input, bool, Marker> + 'static
{
    IntoSystem::into_system(
        system
            .pipe(|In(finish): In<bool>| {
                if finish {
                    Some(())
                } else {
                    None
                }
            })
    )
}

pub fn event<E>() -> impl ReactorSystemConfigs<In=(), Out=E>
    where E: Event + Clone
{
    with((), wait::output(|mut er: EventReader<E>| {
        er.read().next().cloned()
    }))
}


// #[inline]
// pub fn any<
//     Lhs, LIn, LOut, LMark,
//     Rhs, RIn, ROut, RMark
// >(lhs: Lhs, rhs: Rhs) -> impl ReactorSystemConfigs<WithInput, In=((LIn, Lhs), (RIn, Rhs)), Out=()>
//     where
//         Lhs: ReactorSystemConfigs<LMark, In=LIn, Out=LOut> + 'static,
//         Rhs: ReactorSystemConfigs<RMark, In=RIn, Out=ROut> + 'static,
//         LIn: Clone + 'static,
//         LOut: Clone + 'static,
//         RIn: Clone + 'static,
//         ROut: Clone + 'static
// {
//     with(
//         (lhs.into_configs(), rhs.into_configs()),
//         |systems: In<((LIn, Lhs), (RIn, Rhs))>|{
//             
//             Some(())
//         }
//     )
// }

#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{EventWriter, In, Local, Update, World};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{once, wait, with};
    use crate::selector::condition::wait::until;

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(Update, until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 2
                })).await;

                task.will(Update, once::non_send::insert(AppExit)).await;
            });
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

        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(Update, with(1, until(|input: In<u32>, mut count: Local<u32>| {
                    *count += 1 + input.0;
                    *count == 4
                }))).await;

                task.will(Update, once::non_send::insert(AppExit)).await;
            });
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
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let event = task.will(PreUpdate, wait::event::<AppExit>()).await;
                    task.will(Update, once::non_send::insert(event)).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.world.run_system_once(|mut w: EventWriter<AppExit>| w.send(AppExit));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}