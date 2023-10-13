use bevy::prelude::{Event, World};
use futures::{SinkExt, StreamExt};
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystem, AsyncSystemRunnable, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};

#[derive(Default, Debug)]
pub struct SendEvent<E> {
    event: E,
}


impl<E: Event> SendEvent<E> {
    #[inline]
    pub const fn run(event: E) -> SendEvent<E> {
        Self {
            event
        }
    }
}


impl<E: Event + Clone> AsyncSystem<()> for SendEvent<E> {
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<()>) {
        let (tx, mut rx) = new_channel::<()>(1);
        let runner = Box::new(SendRunner {
            sent: false,
            event: self.event,
            tx,
        });

        (runner, Box::pin(async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }))
    }
}


struct SendRunner<E> {
    sent: bool,
    event: E,
    tx: Sender<()>,
}

impl<E: Event + Clone> AsyncSystemRunnable for SendRunner<E> {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if self.sent {
            SystemRunningStatus::Finished
        } else {
            world.send_event(self.event.clone());
            self.sent = true;
            let _ = self.tx.try_send(());
            SystemRunningStatus::Finished
        }
    }
}