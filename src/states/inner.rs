use crate::bot::{Bot, BotConfig};
use rsc2_pb::{prelude::*, sc2_api};

pub trait IsProtocolState {
    fn create_game_request(&self) -> EncodeResult {
        panic!("Invalid Operation");
    }
    fn join_game_request(&self, _: BotConfig) -> EncodeResult {
        panic!("Invalid Operation");
    }
    fn start_replay_request(&self) {
        panic!("Invalid Operation");
    }
    fn gamestate_request(&self, _: u32) -> EncodeResult {
        panic!("Invalid Operation");
    }
    fn restart_game_request(&self) {
        panic!("Invalid Operation");
    }
    fn close_game_request(&self) {
        panic!("Invalid Operation");
    }
}

fn join_game(bot: BotConfig) -> EncodeResult {
    sc2_api::Request::with_id(sc2_api::RequestJoinGame::with_race(bot.race), 1).into()
}

#[derive(Debug, Default)]
pub struct InitGame; // InitGame info
impl IsProtocolState for InitGame {
    fn join_game_request(&self, bot: BotConfig) -> EncodeResult {
        join_game(bot)
    }
}

#[derive(Debug, Default)]
pub struct Launched; // Launched info

impl IsProtocolState for Launched {
    fn create_game_request(&self) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestCreateGame::default_config(), 0).into()
    }
    fn join_game_request(&self, bot: BotConfig) -> EncodeResult {
        join_game(bot)
    }
    fn start_replay_request(&self) {}
}

#[derive(Default)]
pub struct InGame {
    pub bot: Option<Box<dyn Bot>>,
} // InGame info
impl IsProtocolState for InGame {
    fn gamestate_request(&self, game_loop: u32) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestObservation::nofog(game_loop), game_loop).into()
    }
}

#[derive(Debug, Default)]
pub struct InReplay; // InReplay info
impl IsProtocolState for InReplay {
    fn gamestate_request(&self, game_loop: u32) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestObservation::nofog(game_loop), game_loop).into()
    }
}

#[derive(Debug, Default)]
pub struct Ended; // Ended info
impl IsProtocolState for Ended {
    fn restart_game_request(&self) {}
    fn close_game_request(&self) {}
}
