use std::marker::PhantomData;
use std::time::Duration;

use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::{Res, TimerMode};
use bevy::time::{Time, Timer};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::thread_pool::{IntoThreadPoolExecutor, ThreadPoolExecutable, ThreadPoolExecutor};

pub(crate) struct DelayTime(pub Duration);


impl<'w> IntoThreadPoolExecutor<DelayTimeParam<'w>, ()> for DelayTime {
    #[inline]
    fn into_executor(self, sender: Sender<()>) -> ThreadPoolExecutor<DelayTimeParam<'w>> {
        ThreadPoolExecutor::new(Executor {
            timer: Timer::new(self.0, TimerMode::Once),
            sender,
        })
    }
}


struct Executor {
    sender: Sender<()>,
    timer: Timer,
}


impl<'w> ThreadPoolExecutable<DelayTimeParam<'w>> for Executor {
    #[inline]
    fn execute(&mut self, param: &mut StaticSystemParam<DelayTimeParam<'w>>) -> AsyncSystemStatus {
        if self.sender.is_closed() {
            return AsyncSystemStatus::Finished;
        }

        if self.timer.tick(param.time.delta()).just_finished() {
            let _ = self.sender.try_send(());
            AsyncSystemStatus::Finished
        } else {
            AsyncSystemStatus::Running
        }
    }
}


#[derive(SystemParam)]
pub struct DelayTimeParam<'w, Marker: 'static = ()> {
    time: Res<'w, Time>,
    marker: PhantomData<Marker>,
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::app::{App, Update};
    use bevy::core::TaskPoolPlugin;
    use bevy::ecs::system::CommandQueue;
    use bevy::prelude::{Commands, State, States};
    use bevy::time::TimePlugin;

    use crate::AsyncSystemPlugin;
    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::main_thread::once::OnceOnMain;

    use crate::runner::thread_pool::delay::time::DelayTime;

    #[derive(Default, Copy, Clone, Eq, PartialEq, Hash, States, Debug)]
    enum TestState {
        #[default]
        Empty,
        Finished,
    }

    impl TestState {
        fn finished(&self) -> bool {
            matches!(self, TestState::Finished)
        }
    }

    #[test]
    fn delay_time() {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            TimePlugin,
            AsyncSystemPlugin
        ));
        app.add_state::<TestState>();

        let mut command_queue = CommandQueue::default();

        Commands::new(&mut command_queue, &app.world)
            .spawn_async(|cmd| async move {
                cmd.spawn(Update, DelayTime(Duration::ZERO)).await;
                cmd.spawn_on_main(Update, OnceOnMain::set_state(TestState::Finished)).await;
            });

        command_queue.apply(&mut app.world);

        // tick
        app.update();
        // send event
        app.update();
        app.update();

        assert!(app.world
            .resource::<State<TestState>>()
            .finished());
    }
}