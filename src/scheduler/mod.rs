//! Implementations of different scheduling strategies for concurrency testing.
use crate::runtime::task::TaskId;
use std::fmt::Debug;

mod data;
mod dfs;
mod pct;
mod random;
mod replay;
mod round_robin;

pub(crate) mod metrics;
pub(crate) mod serialization;

pub use dfs::DFSScheduler;
pub use pct::PCTScheduler;
pub use random::RandomScheduler;
pub use replay::ReplayScheduler;
pub use round_robin::RoundRobinScheduler;

/// A `Schedule` determines the order in which tasks are to be executed
// TODO would be nice to make this generic in the type of `seed`, but for now all our seeds are u64s
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schedule {
    seed: u64,
    steps: Vec<ScheduleStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ScheduleStep {
    Task(TaskId),
    Random,
}

impl Schedule {
    /// Create a new empty `Schedule` that starts with the given random seed.
    pub fn new(seed: u64) -> Self {
        Self { seed, steps: vec![] }
    }

    /// Create a new `Schedule` that begins by scheduling the given tasks.
    pub fn new_from_task_ids<T>(seed: u64, task_ids: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<TaskId>,
    {
        let steps = task_ids
            .into_iter()
            .map(|t| ScheduleStep::Task(t.into()))
            .collect::<Vec<_>>();
        Self { seed, steps }
    }

    /// Add the given task ID as the next step of the schedule.
    pub fn push_task(&mut self, task: TaskId) {
        self.steps.push(ScheduleStep::Task(task));
    }

    /// Add a choice of a random u64 value as the next step of the schedule.
    pub fn push_random(&mut self) {
        self.steps.push(ScheduleStep::Random);
    }

    /// Return the number of steps in the schedule.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Return true if the schedule is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A `Scheduler` is an oracle that decides the order in which to execute concurrent tasks and the
/// data to return to calls for random values.
///
/// The`Scheduler` lives across multiple executions of the test case, allowing it to retain some
/// state and strategically explore different schedules. At the start of each test execution, the
/// executor calls `new_execution()` to inform the scheduler that a new execution is starting. Then,
/// for each scheduling decision, the executor calls `next_task` to determine which task to run.
pub trait Scheduler: Debug {
    /// Inform the `Scheduler` that a new execution is about to begin. If this function returns
    /// None, the test will end rather than performing another execution. If it returns
    /// `Some(schedule)`, the returned `Schedule` can be used to initialize a `ReplayScheduler` for
    /// deterministic replay.
    fn new_execution(&mut self) -> Option<Schedule>;

    /// Decide which task to run next, given a list of runnable tasks and the currently running
    /// tasks. If `current_task` is `None`, the execution has not yet begun. The list of runnable
    /// tasks is guaranteed to be non-empty.  This method returns `Some(task)` where `task` is
    /// the runnable task to be executed next; it may also return `None`, indicating that the
    /// execution engine should stop exploring the current schedule.
    fn next_task(&mut self, runnable_tasks: &[TaskId], current_task: Option<TaskId>) -> Option<TaskId>;

    /// Choose the next u64 value to return to the currently running task.
    fn next_u64(&mut self) -> u64;
}
