use bevy::app::AppExit;
use bevy::prelude::{Event, EventWriter, In, System};

use crate::selector::condition::{once, ReactorSystemConfigs, with, WithInput};

#[inline]
pub fn send<E>() -> impl System<In=E, Out=Option<()>>
    where E: Event + Clone
{
    once::run(|In(event): In<E>, mut w: EventWriter<E>| {
        w.send(event);
    })
}

#[inline]
pub fn app_exit() -> impl ReactorSystemConfigs<WithInput, In=AppExit, Out=()> {
    with(AppExit, send::<AppExit>())
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{once, with};
    use crate::tests::came_event;

    #[test]
    fn send_event() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, with(AppExit, once::event::send())).await;
                });
            });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
}