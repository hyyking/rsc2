#[cfg(test)]
mod mock {
    use rsc2::api::raw::RawAgent;
    use rsc2::hook::NextRequest;
    use rsc2::pb::api as pb;

    pub(crate) struct MockAgent;

    impl RawAgent for MockAgent {
        fn on_response(&mut self, _: pb::Response) -> NextRequest {
            NextRequest::Observation
        }
    }
}

#[cfg(test)]
mod builder {
    use std::time::Duration;

    use super::mock::MockAgent;

    use rsc2::api::raw::NewRawAgent;
    use rsc2::runtime::{Builder, Coordinator};

    #[test]
    fn test_build() {
        // faster Coordinator that can handle more work
        let _: Coordinator<NewRawAgent<MockAgent>> = Builder::new()
            .interval(Duration::from_micros(100))
            .core_threads(16)
            .build();
    }
}
