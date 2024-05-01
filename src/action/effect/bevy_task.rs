//! Convert the bevy tasks into [`Action`](crate::prelude::Action).
//!
//! actions
//!
//! - [`effect::bevy_task::spawn`](crate::prelude::effect::bevy_task::spawn)
//! - [`effect::bevy_task::spawn_detached`](crate::prelude::effect::bevy_task::spawn_detached)


pub use _spawn::spawn;
pub use _spawn_detached::spawn_detached;

#[path = "bevy_task/spawn.rs"]
mod _spawn;
#[path = "bevy_task/spawn_detached.rs"]
mod _spawn_detached;
