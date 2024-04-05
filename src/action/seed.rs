use crate::action::TaskAction;
use crate::runner::{CancellationToken, TaskOutput, TaskRunner};

pub mod once;
pub mod wait;


pub trait ActionSeed<In = (), Out = ()> {
    fn into_action(self, input: In) -> impl TaskAction<In, Out>;
}

pub trait Seed {}


impl<Out, A> TaskAction<(), Out> for A
    where A: ActionSeed<(), Out> + Seed
{
    #[inline]
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Out>) -> impl TaskRunner + 'static{
        self.into_action(()).to_runner(token, output)
    }
}