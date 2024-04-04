/// Create actions that execute the passed actions in sequence.
/// 
/// It has advantage that if the previous action finishes, 
/// the next will start within in that frame. 
/// 
/// For example, the code below defines three actions, 
/// all of which are executed during one frame.
/// 
/// ```no_run
/// use bevy::app::{App, Update};
/// use bevy::prelude::World;
/// use bevy_flurx::prelude::*;
/// use bevy_flurx::sequence;
///
/// let mut app = App::new();
/// app.world.schedule_reactor(|task| async move{
///     task.will(Update, sequence!{
///         once::run(||{}),
///         once::run(||{}),
///         once::event::app_exit()
///     }).await;
/// });
/// app.update();
/// ```
/// 
#[macro_export]
macro_rules! sequence {
    ($action: expr $(,)?) => {$action};
    ($action1: expr, $action2: expr $(,$action: expr)*$(,)?)  => {
        {
            let a = $crate::private::SequenceRunner::new($crate::action::to_tuple($action1), $action2);
            $(
            let a = $crate::private::SequenceRunner::new(a, $action);
            )*
            a
        }
    };
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{EventWriter, ResMut, Resource, World};
    use bevy_test_helper::event::TestEvent1;

    use crate::action::{once, wait};
    use crate::extension::ScheduleReactor;
    use crate::tests::test_app;

    #[test]
    fn one() {
        let mut app = test_app();
        app
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(Update, sequence! {
                        once::non_send::insert(AppExit)
                    }).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn two() {
        let mut app = test_app();

        app
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(Update, sequence! {
                        wait::event::read::<TestEvent1>(),
                        once::non_send::insert(AppExit)
                    }).await;
                });
            });

        app.world.run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn three() {
        let mut app = test_app();
        #[derive(Resource, Default)]
        struct Count1(usize);

        #[derive(Resource, Default)]
        struct Count2(usize);

        app
            .init_resource::<Count1>()
            .init_resource::<Count2>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(Update, sequence! {
                        once::run(|mut c:  ResMut<Count1>|{
                            c.0 += 1;
                        }),
                        wait::until(|mut c:  ResMut<Count2>|{
                            c.0 += 1;
                            c.0 == 2
                        }),
                        once::non_send::insert(AppExit)
                    }).await;
                });
            });
        app.update();
        assert_eq!(app.world.resource::<Count1>().0, 1);
        assert_eq!(app.world.resource::<Count2>().0, 1);
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert_eq!(app.world.resource::<Count1>().0, 1);
        assert_eq!(app.world.resource::<Count2>().0, 2);
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}



