use crate::proto::sc2_api;

impl Into<sc2_api::request::Request> for sc2_api::RequestPing {
    fn into(self) -> sc2_api::request::Request {
        sc2_api::request::Request::Ping(self)
    }
}

impl Into<sc2_api::request::Request> for sc2_api::RequestCreateGame {
    fn into(self) -> sc2_api::request::Request {
        sc2_api::request::Request::CreateGame(self)
    }
}

impl Into<sc2_api::request::Request> for sc2_api::RequestJoinGame {
    fn into(self) -> sc2_api::request::Request {
        sc2_api::request::Request::JoinGame(self)
    }
}

impl Into<sc2_api::request_create_game::Map> for sc2_api::LocalMap {
    fn into(self) -> sc2_api::request_create_game::Map {
        sc2_api::request_create_game::Map::LocalMap(self)
    }
}
