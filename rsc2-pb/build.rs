extern crate prost_build;

const DERIVE_WRAPENUM: &str = "#[derive(::rsc2_derive::WrapEnum)]";
const DERIVE_TRYINTOENUM: &str = "#[derive(::rsc2_derive::TryIntoEnum)]";

fn main() {
    let mut prost_build = prost_build::Config::new();
    prost_build.btree_map(&["."]);

    // SC2 impl Into<request::Request> for Request.request types
    prost_build.type_attribute("Request.request", DERIVE_WRAPENUM);
    prost_build.type_attribute("Status", DERIVE_TRYINTOENUM);

    prost_build
        .compile_protos(
            &["../s2client-proto/s2clientprotocol/sc2api.proto"],
            &["../s2client-proto"],
        )
        .unwrap();
}
