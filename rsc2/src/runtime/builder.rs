use std::time::Duration;

use crate::hook::AgentHook;
use crate::runtime::Coordinator;

const DEFAULT_CORE_THREADS: usize = 4;
const DEFAULT_INTERVAL: Duration = Duration::from_millis(50);

pub(super) struct CoordinatorConfig {
    pub(super) interval: Duration,
    pub(super) core_threads: usize,
}
impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            core_threads: DEFAULT_CORE_THREADS,
        }
    }
}
impl From<&mut Builder> for CoordinatorConfig {
    fn from(b: &mut Builder) -> Self {
        let interval = b.interval.take().unwrap_or(DEFAULT_INTERVAL);
        let core_threads = b.core_threads.take().unwrap_or(DEFAULT_CORE_THREADS);
        Self {
            interval,
            core_threads,
        }
    }
}

pub struct Builder {
    interval: Option<Duration>,
    core_threads: Option<usize>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            interval: None,
            core_threads: None,
        }
    }
    pub fn build<A: AgentHook + 'static>(&mut self) -> Coordinator<A> {
        Coordinator::from(CoordinatorConfig::from(self))
    }
    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = Some(interval);
        self
    }
    pub fn core_threads(&mut self, core_threads: usize) -> &mut Self {
        self.core_threads = Some(core_threads);
        self
    }
}

#[cfg(test)]
mod builder {

    use crate::api::raw::{NewRawAgent, RawAgent};
    use crate::hook::NextRequest;
    use crate::pb::api as pb;
    use crate::runtime::{Builder, Coordinator};

    use std::time::Duration;

    struct MockAgent {}
    impl RawAgent for MockAgent {
        fn on_response(&mut self, _: pb::Response) -> NextRequest {
            NextRequest::Observation
        }
    }

    #[test]
    fn test_build() {
        // faster Coordinator that can handle more work
        let _: Coordinator<NewRawAgent<MockAgent>> = Builder::new()
            .interval(Duration::from_micros(100))
            .core_threads(16)
            .build();
    }
}
