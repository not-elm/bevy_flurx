use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::prelude::World;
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystemRunnable, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct DelayRunner {
    tx: Sender<()>,
    schedule_label: BoxedScheduleLabel,
    frame_count: usize,
    delay_frames: usize,
}


impl DelayRunner {
    pub fn boxed(
        tx: Sender<()>,
        schedule_label: impl ScheduleLabel,
        delay_frames: usize,
    ) -> BoxedAsyncSystemRunner {
        Box::new(Self {
            tx,
            schedule_label: Box::new(schedule_label),
            frame_count: 0,
            delay_frames,
        })
    }
}


impl AsyncSystemRunnable for DelayRunner {
    fn run(&mut self, _: &mut World) -> SystemRunningStatus {
        self.frame_count += 1;
        if self.frame_count < self.delay_frames {
            SystemRunningStatus::Running
        } else {
            self.tx.try_send(()).unwrap();
            SystemRunningStatus::Finished
        }
    }


    #[inline]
    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool {
        schedule_label.eq(&self.schedule_label)
    }
}