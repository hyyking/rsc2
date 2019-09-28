extern crate prost_build;

fn main() {
    let mut prost_build = prost_build::Config::new();
    prost_build.btree_map(&["."]);

    // SC2 impl Into<request::Request> for Request.request types
    prost_build.type_attribute("", "use rsc2_derive::WrapEnum;");
    prost_build.type_attribute("Request.request", "#[derive(rsc2_derive::WrapEnum)]");

    prost_build
        .compile_protos(
            &["../s2client-proto/s2clientprotocol/sc2api.proto"],
            &["../s2client-proto"],
        )
        .unwrap();
}
