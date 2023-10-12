mod task;
mod system;
mod spawner;

use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Arc, Mutex};

use bevy::{prelude::*};
use bevy::app::{App, Plugin, Update};
use bevy::DefaultPlugins;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{IntoSystem, World};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_async_task::{AsyncReceiver, AsyncTask, AsyncTaskRunner};
use futures::{SinkExt, StreamExt};
use futures::channel::oneshot;
use crate::spawner::TaskManager;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, (setup, world_system))

        .run();
}


fn world_system(mut commands: Commands) {
    let task = AsyncComputeTaskPool::get()
        .spawn(async {
            TaskTest::default().until(|mut tranform: Query<&mut Transform, With<Shape>>| {
                info!("d");
                tranform.single_mut().translation.x += 1.;
                tranform.single().translation.x < 50.
            }).await;
            info!("moved");
        });
    commands.spawn(ComputeTransform(task));
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut tasks: NonSendMut<Tasks>,
) {
    commands.spawn(Camera2dBundle::default());
    // Rectangle
    commands.spawn((
        Shape,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
            ..default()
        }
    ));

    tasks.spawn_async(|mut io| async move {
        info!("spawn_async");
        io.until(|mut tranform: Query<&mut Transform, With<Shape>>| {
            info!("until");
            tranform.single_mut().translation.x += 1.;
            tranform.single().translation.x < 50.
        }).await;
    });

    info!("moved");
}


pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<TaskManager>()
            .add_systems(Update, (
                run_tasks,
            ));
    }
}


fn run_tasks(world: &mut World) {
    let Some(mut tasks) = world.remove_non_send_resource::<TaskManager>() else { return; };
    tasks.running(world);
    world.insert_non_send_resource(tasks);
}

#[derive(Deref, DerefMut, Default)]
struct Tasks(Vec<(AsyncReceiver<()>, TaskTest)>);


impl Tasks {
    pub fn runs(&mut self, world: &mut World) {
        for (rx, task_test) in self.iter_mut() {
            println!("runs");
           let is_finish = task_test.run(world);
            if rx.try_recv().is_some() {
                println!("try_recv");
            }

            // task_test.run(world);
        }
    }
}

#[derive(Component)]
struct ComputeTransform(Task<()>);

pub trait Ext {
    fn spawn_async<T>(&mut self, f: impl Fn(TaskTest) -> T + Send)
        where T: Future<Output=()> + Send + 'static
    ;
}


impl Ext for Tasks {
    fn spawn_async<F>(&mut self, f: impl Fn(TaskTest) -> F + Send)
        where F: Future<Output=()> + Send + 'static
    {
        let io = TaskTest::default();
        let t = f(io.clone());
         let task: AsyncTask<()> = t.into();
        let (fut, rx) = task.into_parts();
        let task_pool = AsyncComputeTaskPool::get();
        let handle = task_pool.spawn(fut);
        handle.detach();

        self.push((rx, io));
    }
}

#[derive(Default)]
pub struct TaskTest {
    callbacks: Arc<Mutex<VecDeque<TaskCallback>>>,
}


impl Clone for TaskTest {
    fn clone(&self) -> Self {
        Self {
            callbacks: Arc::clone(&self.callbacks)
        }
    }
}


impl TaskTest {
    pub async fn until<Marker>(&mut self, callback: impl IntoSystem<(), bool, Marker> + 'static) {
        let (tx, mut rx) = futures::channel::mpsc::channel::<bool>(3);
        println!("until *********** ");
        self.callbacks.lock().unwrap().push_back(TaskCallback::new(tx, callback));
         println!("push_back *********** ");
        while rx.next().await.is_none(){

        }
        println!("locked *********** ");

    }

    pub fn run(&mut self, world: &mut World) -> bool {
        let mut callbacks = self.callbacks.lock().unwrap();
        println!("callbacks = {}", callbacks.len());
        if let Some(callback) = callbacks.front_mut() {
            println!("callback");
            if callback.run(world) {
                callbacks.pop_front();
            }
        }
        callbacks.is_empty()
    }
}


pub struct TaskCallback {
    tx: futures::channel::mpsc::Sender<bool>,
    system: BoxedSystem<(), bool>,
    is_init: bool,
}


impl TaskCallback {
    pub fn new<Marker>(tx: futures::channel::mpsc::Sender<bool>, system: impl IntoSystem<(), bool, Marker> + 'static) -> TaskCallback {
        Self {
            tx,
            system: Box::new(IntoSystem::into_system(system)),
            is_init: false,
        }
    }

    pub fn run(&mut self, world: &mut World) -> bool {
        if !self.is_init {
            self.system.initialize(world);
            self.is_init = true;
        }
  println!("run *********** ");
        let output = self.system.run((), world);
        self.system.apply_deferred(world);
        if !output {
            println!("send *********** ");
            self.tx.try_send(output).unwrap();
        }
        !output
    }
}