use crate::action::Action;
use crate::runner::{CancellationToken, TaskOutput, TaskRunner};

pub mod once;
pub mod wait;


pub trait ActionSeed<In = (), Out = ()> {
    fn with(self, input: In) -> impl Action<In, Out>;
}

pub trait Seed {}


impl<Out, A> Action<(), Out> for A
    where A: ActionSeed<(), Out> + Seed
{
    #[inline]
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Out>) -> impl TaskRunner + 'static{
        self.with(()).to_runner(token, output)
    }
}