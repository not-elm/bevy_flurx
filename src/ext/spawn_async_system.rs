use std::future::Future;
use async_compat::CompatExt;
use async_trait::async_trait;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use bevy::tasks::AsyncComputeTaskPool;
use crate::async_commands::{AsyncSchedules, TaskHandle};

#[async_trait]
pub trait SpawnAsyncSystem<'w, 's> {
    /// Build an asynchronous system.
    ///
    /// [`AsyncSchedules`] is passed as an argument and is used to declare the system.
    ///
    /// ## Notes
    ///
    /// The way the scheduler executes is different from normal systems.
    /// For example, normally if you add a systems with [`Update`], it will be executed every frame,
    /// but this is not the necessarily case with `async_system`.
    /// 'async_system' needs to pass the system and the conditions under which its task will finish.
    ///
    ///
    /// ## Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_async_system::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.add_event::<TestEvent>();
    /// app.add_plugins((
    ///     TaskPoolPlugin::default(),
    ///     AsyncSystemPlugin
    /// ));
    ///
    /// app.add_systems(Startup, |mut commands: Commands|{
    ///     commands.spawn_async(|schedules|async move{
    ///         // `once` systems...
    ///         schedules.add_system(Update, once::run(print_system)).await;
    ///         let output = schedules.add_system(Update, once::run(return_count)).await;
    ///         schedules.add_system(Update, once::insert_resource(output)).await;
    ///
    ///         // `wait` systems...
    ///         schedules.add_system(Update, wait::until(count_up_to_5)).await;
    ///         schedules.add_system(Update, wait::until_event::<TestEvent>()).await;
    ///
    ///         // repeat systems...
    ///         schedules.add_system(Update, repeat::times(2, count_up)).await;
    ///         let handle = schedules.add_system(Update, repeat::forever(count_up));
    ///
    ///         // delay systems...
    ///         schedules.add_system(Update, delay::frames(3)).await;
    ///         drop(handle);
    ///         schedules.add_system(Update, once::run(|mut count: ResMut<Count>|{
    ///             count.0 -= 1;
    ///         })).await;
    ///     });
    /// });
    ///
    /// app.update(); // run `println_system`
    /// app.update(); // run `output_system`
    /// app.update(); // run `output`
    /// assert_eq!(app.world.resource::<Count>(), &Count(3));
    ///
    /// app.update(); // run `count_up_to_5` count = 4
    /// assert_eq!(app.world.resource::<Count>(), &Count(4));
    /// app.update(); // run `count_up_to_5` count = 5
    /// assert_eq!(app.world.resource::<Count>(), &Count(5));
    /// app.update(); // run `until_event::<TestEvent>`
    /// app.world.send_event(TestEvent);
    /// app.update(); // run `until_event::<TestEvent>`
    ///
    /// app.update(); // run `count_up`
    /// assert_eq!(app.world.resource::<Count>(), &Count(6));
    /// app.update(); // run `count_up`
    /// assert_eq!(app.world.resource::<Count>(), &Count(7));
    ///
    /// app.update(); // run `count_up` and `delay::frames(3)`
    /// assert_eq!(app.world.resource::<Count>(), &Count(8));
    /// app.update(); // run `count_up` and `delay::frames(3)`
    /// assert_eq!(app.world.resource::<Count>(), &Count(9));
    /// app.update(); // run `count_up` and `delay::frames(3)`
    /// assert_eq!(app.world.resource::<Count>(), &Count(10));
    ///
    /// app.update();
    /// assert_eq!(app.world.resource::<Count>(), &Count(9));
    /// fn print_system(){
    ///     println!("Hello world!");
    /// }
    ///
    /// #[derive(Clone, Resource, Eq, PartialEq, Debug)]
    /// struct Count(usize);
    ///
    /// #[derive(Event)]
    /// struct TestEvent;
    ///
    /// fn return_count() -> Count{
    ///     Count(3)
    /// }
    ///
    /// fn count_up_to_5(mut count: ResMut<Count>) -> bool{
    ///     count.0 +=1;
    ///     count.0 == 5
    /// }
    ///
    /// fn count_up(mut count: ResMut<Count>){
    ///     count.0 +=1;
    /// }
    /// ```
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;


    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + 'static;
}


impl<'w, 's> SpawnAsyncSystem<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let async_commands = AsyncSchedules::default();
        let handle = AsyncComputeTaskPool::get().spawn(f(async_commands.clone()).compat());

        self.spawn((
            async_commands.schedulers,
            TaskHandle(handle)
        ))
    }


    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + 'static {
        let async_commands = AsyncSchedules::default();
        let handle = AsyncComputeTaskPool::get().spawn_local(f(async_commands.clone()).compat());

        self.spawn((
            async_commands.schedulers,
            TaskHandle(handle)
        ))
    }
}




