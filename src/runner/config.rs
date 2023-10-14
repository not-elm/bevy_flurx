use bevy::ecs::system::BoxedSystem;
use bevy::prelude::IntoSystem;

pub(crate) struct AsyncSystemConfig<Out = ()> {
    pub system: BoxedSystem<(), Out>,
}


impl<Out> AsyncSystemConfig<Out> {
    #[inline]
    pub fn new<Marker>(system: impl IntoSystem<(), Out, Marker> + 'static + Send) -> AsyncSystemConfig<Out> {
        Self {
            system: Box::new(IntoSystem::into_system(system))
        }
    }
}


