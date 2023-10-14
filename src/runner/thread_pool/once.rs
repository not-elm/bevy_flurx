use std::sync::{Arc, Mutex};

use bevy::ecs::system::{StaticSystemParam, SystemParam};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::thread_pool::{IntoThreadPoolExecutor, ThreadPoolExecutable, ThreadPoolExecutor};

pub type SharedCallback<Param, Out> = Arc<Mutex<dyn Fn(&mut StaticSystemParam<Param>) -> Out>>;


pub struct OnceOnThread<Param: SystemParam, Out>(pub SharedCallback<Param, Out>);

unsafe impl<Param: SystemParam, Out> Send for OnceOnThread<Param, Out> {}


unsafe impl<Param: SystemParam, Out> Sync for OnceOnThread<Param, Out> {}


impl<Param: SystemParam + 'static, Out: 'static> OnceOnThread<Param, Out> {
    #[inline(always)]
    pub fn run(f: impl Fn(&mut StaticSystemParam<Param>) -> Out + Send + 'static) -> impl IntoThreadPoolExecutor<Param, Out> {
        OnceOnThread(Arc::new(Mutex::new(f)))
    }
}


impl<Param: SystemParam + 'static, Out: 'static> IntoThreadPoolExecutor<Param, Out> for OnceOnThread<Param, Out> {
    #[inline]
    fn into_executor(self, sender: Sender<Out>) -> ThreadPoolExecutor<Param> {
        ThreadPoolExecutor::new(Executor {
            callback: self.0,
            sender,
        })
    }
}


struct Executor<Param: SystemParam, Out> {
    callback: SharedCallback<Param, Out>,
    sender: Sender<Out>,
}


impl<Param: SystemParam, Out> ThreadPoolExecutable<Param> for Executor<Param, Out> {
    fn execute(&mut self, param: &mut StaticSystemParam<Param>) -> AsyncSystemStatus {
        if self.sender.is_closed() {
            return AsyncSystemStatus::Finished;
        }

        let output = self.callback.lock().unwrap()(param);
        let _ = self.sender.try_send(output);
        AsyncSystemStatus::Finished
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::ecs::system::StaticSystemParam;
    use bevy::prelude::{Commands, Events, EventWriter};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;

    use crate::runner::thread_pool::once::OnceOnThread;
    use crate::test_util::{new_app, TestEvent};

    #[test]
    fn once_on_thread_pool() {
        let mut app = new_app();
        app.add_systems(Startup, setup);
        app.update();

        let er = ManualEventReader::<TestEvent>::default();
        let events = app.world.resource::<Events<TestEvent>>();
        assert!(!er.is_empty(events));
    }


    fn setup(mut commands: Commands) {
        commands.spawn_async(|cmd| async move {
            cmd.spawn(Update, OnceOnThread::run(send_event)).await;
        });
    }


    fn send_event(param: &mut StaticSystemParam<EventWriter<TestEvent>>) {
        param.send(TestEvent);
    }
}