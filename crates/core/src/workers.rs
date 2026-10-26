/// The port for running work side by side, such as the code engines of a build. The adapter
/// decides on how many threads, if any; the core only hands it independent tasks.
pub trait Workers: Send + Sync {
    /// Calls `task` once for every index below `count`, in any order and possibly at the same
    /// time, and returns when every call has returned.
    fn run(&self, count: usize, task: &(dyn Fn(usize) + Sync));
}

/// Runs every task in turn, for hosts without threads and projects without workers.
pub(crate) struct InTurn;

impl Workers for InTurn {
    fn run(&self, count: usize, task: &(dyn Fn(usize) + Sync)) {
        (0..count).for_each(task);
    }
}
