use std::cell::RefCell;
use std::time::Duration;

use crate::hook::AgentHook;
use crate::runtime::Coordinator;

use tokio::runtime::{Builder as rtBuilder, Runtime};

const DEFAULT_INTERVAL: Duration = Duration::from_millis(50);
const DEFAULT_CORE_THREADS: usize = 4;

fn default_runtime() -> Runtime {
    rtBuilder::new()
        .enable_all()
        .threaded_scheduler()
        .thread_name("rsc2-runtime")
        .num_threads(DEFAULT_CORE_THREADS)
        .build()
        .unwrap()
}

pub(super) struct CoordinatorConfig {
    pub(super) interval: Duration,
    pub(super) runtime: RefCell<Runtime>,
}
impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            runtime: RefCell::new(default_runtime()),
        }
    }
}
impl From<&mut Builder> for CoordinatorConfig {
    fn from(b: &mut Builder) -> Self {
        let interval = b.interval.take().unwrap_or(DEFAULT_INTERVAL);
        let runtime = RefCell::new(b.runtime.take().unwrap_or(default_runtime()));
        Self { interval, runtime }
    }
}

/// Builder interface to get a new [`Coordinator`](crate::runtime::Coordinator).
#[derive(Debug)]
pub struct Builder {
    interval: Option<Duration>,
    runtime: Option<Runtime>,
}

impl Builder {
    /// New builder instance
    pub fn new() -> Self {
        Self {
            interval: None,
            runtime: None,
        }
    }
    /// Return the new [`Coordinator`](crate::runtime::Coordinator). Builder state is reset after this
    /// call.
    pub fn build<A: AgentHook + 'static>(&mut self) -> Coordinator<A> {
        Coordinator::from(CoordinatorConfig::from(self))
    }
    /// Interval between game loops. Default value beeing 50ms
    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = Some(interval);
        self
    }
    /// Set a custom runtime for the [`Coordinator`](crate::runtime::Coordinator) to use.
    pub fn runtime(&mut self, runtime: Runtime) -> &mut Self {
        self.runtime = Some(runtime);
        self
    }
}
