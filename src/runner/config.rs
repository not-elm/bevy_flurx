use bevy::ecs::system::BoxedSystem;
use bevy::prelude::IntoSystem;

pub(crate) struct AsyncSystemConfig<In = (), Out = ()> {
    pub input: In,
    pub system: BoxedSystem<In, Out>,
}


impl<In, Out> AsyncSystemConfig<In, Out> {
    pub fn with_input<Marker>(input: In, system: impl IntoSystem<In, Out, Marker> + 'static + Send) -> AsyncSystemConfig<In, Out> {
        Self {
            input,
            system: Box::new(IntoSystem::into_system(system)),
        }
    }
}


impl<Out> AsyncSystemConfig<(), Out> {
    #[inline]
    pub fn new<Marker>(system: impl IntoSystem<(), Out, Marker> + 'static + Send) -> AsyncSystemConfig<(), Out> {
        Self::with_input((), system)
    }
}





#[macro_export]
macro_rules! impl_async_runner_constructor {
    ($name: ident) => {
        impl<In, Out> $name<In, Out> {
            pub fn with_input<Marker>(input: In, system: impl bevy::prelude::IntoSystem<In, Out, Marker> + 'static + Send) -> $name<In, Out> {
                Self(AsyncSystemConfig::with_input(input, system))
            }
        }


        impl <Out> $name<(), Out> {
            #[inline]
            pub fn run<Marker>(system: impl bevy::prelude::IntoSystem<(), Out, Marker> + 'static + Send) -> $name<(), Out>{
                Self::with_input((), system)
            }
        }
    };
}