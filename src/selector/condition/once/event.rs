use bevy::app::AppExit;
use bevy::prelude::{Event, EventWriter, In};

use crate::selector::condition::{once, ReactorSystemConfigs, with};

#[inline]
pub fn send<E>(event: E) -> impl ReactorSystemConfigs<In=E>
    where E: Event + Clone
{
    with(event, once::run(|In(event): In<E>, mut w: EventWriter<E>| {
        w.send(event);
    }))
}

#[inline]
pub fn app_exit() -> impl ReactorSystemConfigs<In=AppExit> {
    send(AppExit)
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::once;
    use crate::tests::came_event;

    #[test]
    fn send_event() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, once::event::send(AppExit)).await;
                });
            });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
}