use bevy::prelude::World;

use crate::action::Action;
use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

#[inline]
pub fn omit<I, O>(action: impl Into<Action<I, O>> + 'static) -> Action<I, ()>
    where
        I: 'static,
        O: 'static
{
    let Action(input, seed) = action.into();
    ActionSeed::new(|input, token, output| {
        let o1 = Output::default();
        let r1 = seed.create_runner(input, token.clone(), o1.clone());
        OmitRunner {
            token,
            output,
            o1,
            r1,
        }
    })
        .with(input)
}

pub trait Omit<I, O> {
    /// Create an action that converts the output 
    /// the action from `O` to `()`.
    /// 
    /// This method is useful for defining groups of actions.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    /// 
    /// Reactor::schedule(|task|async move{
    ///     task.will(Update, play_audio()
    ///         .then(once::run(||{
    ///             println!("the audio has been finished!");
    ///         }))
    ///     ).await;   
    /// });
    /// 
    /// fn play_audio() -> Action<(&'static str, PlaybackSettings), ()>{
    ///     once::audio::play().with(("example.ogg", PlaybackSettings::default()))
    ///         .pipe(wait::audio::finished())
    ///         .omit()
    /// }
    /// ```
    fn omit(self) -> Action<I, ()>;
}

impl<I, O, A> Omit<I, O> for A
    where
        A: Into<Action<I, O>> + 'static,
        I: 'static,
        O: 'static
{
    #[inline]
    fn omit(self) -> Action<I, ()> {
        omit(self)
    }
}

struct OmitRunner<O> {
    token: CancellationToken,
    output: Output<()>,
    r1: BoxedRunner,
    o1: Output<O>,
}

impl<O> Runner for OmitRunner<O> {
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }
        self.r1.run(world);
        if self.o1.is_some() {
            self.output.replace(());
            true
        } else {
            false
        }
    }
}