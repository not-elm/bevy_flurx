pub mod non_send;
pub mod multi_thread;


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