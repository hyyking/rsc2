use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

use crate::hook::AgentHook;
use crate::pb::api as pb;

use futures::lock::Mutex;
use futures::stream::{SplitSink, SplitStream};
use rsc2_pb::codec::SC2ProtobufClient;
use tokio::runtime::Handle;
use tokio::time::Interval;

type StreamPart = SplitStream<SC2ProtobufClient>;
type SinkPart = SplitSink<SC2ProtobufClient, pb::Request>;

pub(super) struct InGameRessource<'a, A: AgentHook + 'static> {
    pub(super) main: MainRessource<'a>,
    pub(super) reqr: Arc<RequestRessource<A>>,
    pub(super) resr: Arc<ResponseRessource>,
    pub(super) ager: Arc<AgentRessource<A>>,
}

pub(super) struct MainRessource<'a> {
    pub(super) rt: &'a Handle,        // Main loop
    pub(super) timer: Interval,       // Main loop
    pub(super) lock: Arc<AtomicBool>, // Main loop | Agent loop
}

pub(super) struct RequestRessource<A: AgentHook + 'static> {
    pub(super) count: Arc<AtomicU32>,    // Request loop | Response loop
    pub(super) sink: Mutex<SinkPart>,    // Main loop once | Request loop
    pub(super) prod: Mutex<A::Producer>, // Request loop
}

pub(super) struct ResponseRessource {
    pub(super) count: Arc<AtomicU32>,     // Request loop | Response loop
    pub(super) stream: Mutex<StreamPart>, // Response loop
}

pub(super) struct AgentRessource<A: AgentHook + 'static> {
    pub(super) lock: Arc<AtomicBool>, // Main loop | Agent loop
    pub(super) agent: Mutex<A>,       // Main loop once | Agent loop
}
