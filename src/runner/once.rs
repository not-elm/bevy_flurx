pub mod on_main {
    pub use crate::runner::main_thread::once::{
        run,
        send,
        set_state,
    };
}


// pub mod on_thread {
//     pub use crate::runner::thread_pool::once::{
//         run,
//         send,
//         set_state,
//     };
// }
