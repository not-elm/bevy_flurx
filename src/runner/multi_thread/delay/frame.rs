use bevy::ecs::system::{StaticSystemParam, SystemParam};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::multi_thread::{IntoThreadPoolExecutor, ThreadPoolExecutable, ThreadPoolExecutor};

pub(crate) struct DelayFrame(pub usize);


impl IntoThreadPoolExecutor<DelayFrameParam, ()> for DelayFrame {
    #[inline]
    fn into_executor(self, sender: Sender<()>) -> ThreadPoolExecutor<DelayFrameParam> {
        ThreadPoolExecutor::new(Executor {
            current_ticks: 0,
            delay_frames: self.0,
            sender,
        })
    }
}


struct Executor {
    current_ticks: usize,
    delay_frames: usize,
    sender: Sender<()>,
}



#[derive(SystemParam)]
pub struct DelayFrameParam;

impl ThreadPoolExecutable<DelayFrameParam> for Executor {
    fn execute(&mut self, _: &mut StaticSystemParam<DelayFrameParam>) -> AsyncSystemStatus {
        self.current_ticks += 1;

        if self.current_ticks < self.delay_frames {
            AsyncSystemStatus::Running
        } else {
            let _ = self.sender.try_send(());
            AsyncSystemStatus::Finished
        }
    }
}