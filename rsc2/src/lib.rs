// pub(crate) mod stream;

pub mod api;
pub mod hook;

pub mod runtime;

pub mod pb {
    pub use rsc2_pb::api;
    pub use rsc2_pb::prelude;
}

pub use rsc2_macro::run;
