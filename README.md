# *RSC2*

A runtime for executing sc2-api agents

## Release

There is currently no plan for release this crate to [`crates.io`](https://crate.io) for the moment as the code and the api needs to be cleaned up.

## Contributing

- Open an issue if you want anything to change
- Pull requests are welcome

## Example

You need to have a Starcraft II instance running to use this API atm.

```rust
// "Hello World" spamming agent

use rsc2::api::raw::{NewRawAgent, RawAgent, RawRequestGame};
use rsc2::hook::NextRequest;
use rsc2::pb::{api, prelude::*};

use std::pin::Pin;

struct Bot;

impl RawAgent for Bot {
    fn on_start(self: Pin<&mut Self>, _: api::Response) -> NextRequest {
        NextRequest::Observation
    }
    fn on_response(self: Pin<&mut Self>, _: api::Response) -> NextRequest {
        let actions = vec![api::Action {
            action_raw: None,
            action_feature_layer: None,
            action_render: None,
            action_ui: None,
            action_chat: Some(api::ActionChat {
                channel: Some(1),
                message: Some("Hello World".into()),
            }),
            game_loop: Some(0),
        }];
        NextRequest::Agent(api::request::Request::Action(api::RequestAction {
            actions,
        }))
    }
    fn on_end(&mut self) {}
}

#[rsc2::run]
fn main() -> std::io::Result<u32> {
    RawRequestGame::new(
        NewRawAgent(Bot {}),
        api::RequestCreateGame::default_config(),
        api::RequestJoinGame::default_config(),
    )
}
