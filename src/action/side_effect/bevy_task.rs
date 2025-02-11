//! Convert the bevy tasks into [`Action`](crate::prelude::Action).
//!
//! actions
//!
//! - [`side_effect::bevy_task::spawn`](crate::prelude::side_effect::bevy_task::spawn)
//! - [`side_effect::bevy_task::spawn_detached`](crate::prelude::side_effect::bevy_task::spawn_detached)

pub use _spawn::spawn;
pub use _spawn_detached::spawn_detached;

#[path = "bevy_task/spawn.rs"]
mod _spawn;
#[path = "bevy_task/spawn_detached.rs"]
mod _spawn_detached;
