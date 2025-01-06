use crate::action::{Action, Map};
use crate::prelude::{ActionSeed, Omit, OmitInput};
use crate::runner::{BoxedRunner, Output};
use std::marker::PhantomData;

/// Represents the track of act.
pub struct Track<Act> {
    /// Represents the act to be recorded.
    pub act: Act,

    /// Represents the process called when a rollback is request for the `Act`
    /// held this structure.
    pub rollback: Rollback,
}

impl<Act> Track<Act>
where
    Act: Send + Sync + 'static,
{
    #[inline]
    pub(crate) fn create_runner(&self, output: Output<Option<ActionSeed>>) -> BoxedRunner {
        (self.rollback.0)().create_runner(output)
    }
}


/// This structure holds the function that will be called when an `undo` operation is requested on the track that holds it.
#[repr(transparent)]
pub struct Rollback(Box<dyn Fn() -> Action<(), Option<ActionSeed>> + Send + Sync>);

impl Rollback {
    /// Create a [`Rollback`] with the function creates `undo action`.
    ///
    /// Its action's output is [`Option<RedoAction>`], which for [`Option::Some`] creates a `redo action`
    /// and pushes it onto the `redo stack`. In the case of [`Option::None`], only `undo action` is performed
    /// and no `redo action` is created.
    ///
    /// This method is typically used when you want to create [`RedoAction`] under certain conditions.
    ///
    /// # Examples
    ///
    /// ```no_run
    ///
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// #[derive(Event)]
    /// struct MyEvent;
    ///
    /// Rollback::new(|| once::run(|mut er: EventReader<MyEvent>|{
    ///     er.is_empty().then_some(RedoAction(once::run(||{})))
    /// }));
    /// ```
    pub fn new<I, A, F>(f: F) -> Self
    where
        I: 'static,
        F: Fn() -> A + Send + Sync + 'static,
        A: Into<Action<I, Option<RedoAction>>> + Send + Sync + 'static,
    {
        Self(Box::new(move || { f().omit_input().map(|redo| redo.map(|r| r.0)).with(()) }))
    }

    /// Create a [`Rollback`] with the function creates `undo action`.
    ///
    /// An `undo action` created by this method doesn't create a `redo action`.
    ///
    /// # Examples
    ///
    /// ```no_run
    ///
    /// use bevy_flurx::prelude::*;
    ///
    /// Rollback::undo(|| once::run(||{}));
    /// ```
    pub fn undo<I, O, A, F>(f: F) -> Self
    where
        I: 'static,
        O: 'static,
        F: Fn() -> A + Send + Sync + 'static,
        A: Into<Action<I, O>> + Send + Sync + 'static,
    {
        Self(Box::new(move || { f().omit_input().map(|_| None).with(()) }))
    }

    /// Create a Restore with the function creates undo action.
    /// Its action need to return the [`RedoAction`] as output.
    ///
    /// # Examples
    ///
    /// ```no_run
    ///
    /// use bevy_flurx::prelude::*;
    ///
    /// Rollback::undo_redo(|| once::run(||{
    ///     RedoAction::new(once::run(||{
    ///
    ///     }))
    /// }));
    /// ```
    pub fn undo_redo<I, A, F>(f: F) -> Self
    where
        I: 'static,
        F: Fn() -> A + Send + Sync + 'static,
        A: Into<Action<I, RedoAction>> + Send + Sync + 'static,
    {
        Self(Box::new(move || { f().omit_input().map(|redo| Some(redo.0)).with(()) }))
    }

    /// Declare undo and redo separately.
    ///
    /// [`Undo`] is passed a function that creates an action to perform `undo`.
    ///
    /// [`Redo`] is passed a function that creates an action to perform `redo`.
    /// The input of its function is the output of `undo action`.
    ///
    /// If you do not want to generate redo, pass [`Redo::NONE`].
    ///
    /// # Examples
    ///
    /// ```no_run
    ///
    /// use bevy::prelude::In;
    /// use bevy_flurx::prelude::*;
    ///
    /// Rollback::parts(
    ///     Undo::make(|| once::run(|| 3u8)),
    ///     Redo::make(|num: u8| { // num is the output of `Undo action`
    ///         once::run(|In(num): In<u8>|{
    ///             assert_eq!(num, 3);
    ///         })
    ///             .with(num)
    ///     })
    /// );
    /// Rollback::parts(
    ///     Undo::make(|| once::run(|| {})),
    ///     Redo::NONE
    /// );
    /// ```
    pub fn parts<
        I, O, A, F,
        RI, RO, RA, RF,
    >(
        undo: Undo<F, I, O, A>,
        redo: (Option<RF>, PhantomData<(RI, O, RO, RA)>),
    ) -> Self
    where
        I: Send + Sync + 'static,
        O: 'static,
        F: Fn() -> A + Send + Sync + 'static,
        A: Into<Action<I, O>> + Send + Sync + 'static,
        RI: Send + Sync + 'static,
        RF: Fn(O) -> RA + Clone + Send + Sync + 'static,
        RO: Send + Sync + 'static,
        RA: Into<Action<RI, RO>> + Send + Sync + 'static,
    {
        let undo = undo.0;
        let redo = redo.0;
        Self::new(move || {
            let redo = redo.clone();
            undo()
                .into()
                .map(move |o| {
                    redo.as_ref().map(|redo| RedoAction::new((redo)(o)))
                })
        })
    }
}

/// This action is executed when one of the [`record::redo`](crate::prelude::record::redo) is called.
#[repr(transparent)]
pub struct RedoAction(pub ActionSeed);

impl RedoAction {
    /// Creates the [`RedoAction`].
    #[inline]
    pub fn new<I, O>(action: impl Into<Action<I, O>> + Send + Sync + 'static) -> Self
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        Self(action.into().omit())
    }
}

/// One of the args passed to [`Restore::parts`](Rollback::parts).
pub struct Undo<F, I, O, A>(F, PhantomData<(I, O, A)>);

impl<F, I, O, A> Undo<F, I, O, A> {
    /// Creates an `undo` action generator.
    pub const fn make(f: F) -> Self {
        Self(f, PhantomData)
    }
}

/// One of the args passed to [`Restore::parts`](Rollback::parts).
pub struct Redo<I>(PhantomData<I>);

impl<I> Redo<I> {
    /// Does not generate `redo action`.
    pub const NONE: (Option<fn(I) -> ActionSeed<I, ()>>, PhantomData<((), I, (), ActionSeed)>) = (None, PhantomData::<((), I, (), ActionSeed)>);


    /// Check [`Restore::parts`](Rollback::parts).
    pub const fn make<F, RI, O, A>(f: F) -> (Option<F>, PhantomData<(RI, I, O, A)>) {
        (Some(f), PhantomData)
    }
}


