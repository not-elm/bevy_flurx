pub mod main_thread;
pub mod thread_pool;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AsyncSystemStatus {
    Running,
    Finished,
}


impl AsyncSystemStatus {
    #[inline(always)]
    pub const fn is_running(&self) -> bool {
        matches!(self, AsyncSystemStatus::Running)
    }


    #[inline(always)]
    pub const fn finished(&self) -> bool {
        matches!(self, AsyncSystemStatus::Finished)
    }
}