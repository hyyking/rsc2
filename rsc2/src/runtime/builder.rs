use std::time::Duration;

use crate::hook::AgentHook;
use crate::runtime::Coordinator;

use tokio::runtime::{Builder as rtBuilder, Runtime};

const DEFAULT_INTERVAL: Duration = Duration::from_millis(50);
const DEFAULT_CORE_THREADS: usize = 4;

pub(super) struct CoordinatorConfig {
    pub(super) interval: Duration,
    pub(super) runtime: Runtime,
}
impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            runtime: Runtime::new().unwrap(),
        }
    }
}
impl From<&mut Builder> for CoordinatorConfig {
    fn from(b: &mut Builder) -> Self {
        let interval = b.interval.take().unwrap_or(DEFAULT_INTERVAL);
        let runtime = b.runtime.take().unwrap_or({
            rtBuilder::new()
                .name_prefix("rsc2-worker-")
                .core_threads(DEFAULT_CORE_THREADS)
                .build()
                .unwrap_or(Runtime::new().unwrap())
        });
        Self { interval, runtime }
    }
}

pub struct Builder {
    interval: Option<Duration>,
    runtime: Option<Runtime>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            interval: None,
            runtime: None,
        }
    }
    pub fn build<A: AgentHook + 'static>(&mut self) -> Coordinator<A> {
        Coordinator::from(CoordinatorConfig::from(self))
    }
    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = Some(interval);
        self
    }
    pub fn runtime(&mut self, runtime: Runtime) -> &mut Self {
        self.runtime = Some(runtime);
        self
    }
}
