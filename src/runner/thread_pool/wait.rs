use bevy::ecs::system::{StaticSystemParam, SystemParam};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::thread_pool::{IntoThreadPoolExecutor, SharedCallback, ThreadPoolExecutable, ThreadPoolExecutor};

#[inline(always)]
pub fn output<Param: SystemParam + 'static, Out: 'static>(
    f: impl Fn(&mut StaticSystemParam<Param>) -> Option<Out> + Send + 'static
) -> impl IntoThreadPoolExecutor<Param, Out> {
    WaitOnThread(SharedCallback::new(f))
}


struct WaitOnThread<Param: SystemParam + 'static, Out: 'static>(SharedCallback<Param, Option<Out>>);


impl<Param: SystemParam + 'static, Out: 'static> IntoThreadPoolExecutor<Param, Out> for WaitOnThread<Param, Out> {
    #[inline]
    fn into_executor(self, sender: Sender<Out>) -> ThreadPoolExecutor<Param> {
        ThreadPoolExecutor::new(Executor {
            sender,
            callback: self.0,
        })
    }
}


struct Executor<Param: SystemParam + 'static, Out: 'static> {
    callback: SharedCallback<Param, Option<Out>>,
    sender: Sender<Out>,
}


impl<Param: SystemParam + 'static, Out: 'static> ThreadPoolExecutable<Param> for Executor<Param, Out> {
    fn execute(&mut self, param: &mut StaticSystemParam<Param>) -> AsyncSystemStatus {
        if self.sender.is_closed() {
            return AsyncSystemStatus::Finished;
        }
        if let Some(output) = self.callback.0.lock().unwrap()(param) {
            let _ = self.sender.try_send(output);
            AsyncSystemStatus::Running
        } else {
            AsyncSystemStatus::Running
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::core::FrameCount;
    use bevy::ecs::event::ManualEventReader;
    use bevy::ecs::system::StaticSystemParam;
    use bevy::prelude::{Commands, Res};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{once, wait};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn wait_output() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                let output = cmd.spawn(Update, wait::on_thread::output(frame_count)).await;
                if output == 1 {
                    cmd.spawn(Update, once::on_thread::send(FirstEvent)).await;
                }
            });
        });
        app.update();
        app.update();
        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut ManualEventReader::default()));
    }


    fn frame_count(frame_count: &mut StaticSystemParam<Res<FrameCount>>) -> Option<u32> {
        if frame_count.0 == 0 {
            None
        } else {
            Some(frame_count.0)
        }
    }
}