use std::sync::{Arc, Mutex};

use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::{Event, EventWriter, NextState, ResMut, States};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::thread_pool::{IntoThreadPoolExecutor, ThreadPoolExecutable, ThreadPoolExecutor};

pub struct OnceOnThread<Param: SystemParam, Out>(pub SharedCallback<Param, Out>);


#[inline(always)]
pub fn run<Param: SystemParam + 'static, Out: 'static>(f: impl Fn(&mut StaticSystemParam<Param>) -> Out + Send + 'static) -> impl IntoThreadPoolExecutor<Param, Out> {
    OnceOnThread(Arc::new(Mutex::new(f)))
}


#[inline(always)]
pub fn send<E: Event + Clone>(event: E) -> impl IntoThreadPoolExecutor<EventWriter<'static, E>, ()> {
    run(move |ew: &mut StaticSystemParam<EventWriter<E>>| {
        ew.send(event.clone());
    })
}


#[inline(always)]
pub fn set_state<S: States + Copy>(state: S) -> impl IntoThreadPoolExecutor<ResMut<'static, NextState<S>>, ()> {
    run(move |next_state: &mut StaticSystemParam<ResMut<NextState<S>>>| {
        next_state.set(state);
    })
}


pub type SharedCallback<Param, Out> = Arc<Mutex<dyn Fn(&mut StaticSystemParam<Param>) -> Out>>;


unsafe impl<Param: SystemParam, Out> Send for OnceOnThread<Param, Out> {}


unsafe impl<Param: SystemParam, Out> Sync for OnceOnThread<Param, Out> {}


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
    use bevy::prelude::{Commands, EventWriter};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::once;
    use crate::test_util::{FirstEvent, is_first_event_already_coming, is_second_event_already_coming, new_app, SecondEvent, test_state_finished, TestState};

    #[test]
    fn once_on_thread_pool() {
        let mut app = new_app();
        app.add_systems(Startup, setup);
        app.update();
        let mut er_first = ManualEventReader::default();
        assert!(is_first_event_already_coming(&mut app, &mut er_first));
    }

    #[test]
    fn once_send() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                cmd.spawn(Update, once::on_thread::send(FirstEvent)).await;
            });
        });
        app.update();
        let mut er_first = ManualEventReader::default();
        assert!(is_first_event_already_coming(&mut app, &mut er_first));
    }


    #[test]
    fn once_set_state() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                cmd.spawn(Update, once::on_thread::set_state(TestState::Finished)).await;
            });
        });
        app.update();
        app.update();

        assert!(test_state_finished(&mut app));
    }

    #[test]
    fn once_called_never_be_called_again() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                cmd.spawn(Update, once::on_thread::send(FirstEvent)).await;
                cmd.spawn(Update, once::on_thread::send(SecondEvent)).await;
            });
        });
        let mut er_first = ManualEventReader::default();
        let mut er_second = ManualEventReader::default();
        app.update();
        assert!(is_first_event_already_coming(&mut app, &mut er_first));
        assert!(!is_second_event_already_coming(&mut app, &mut er_second));

        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er_first));
        assert!(is_second_event_already_coming(&mut app, &mut er_second));

        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er_first));
        assert!(!is_second_event_already_coming(&mut app, &mut er_second));
    }


    fn setup(mut commands: Commands) {
        commands.spawn_async(|cmd| async move {
            cmd.spawn(Update, once::on_thread::run(send_event)).await;
        });
    }


    fn send_event(param: &mut StaticSystemParam<EventWriter<FirstEvent>>) {
        param.send(FirstEvent);
    }
}