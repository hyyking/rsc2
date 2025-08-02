fn build_protos() {
    let mut prost_build = prost_build::Config::new();
    prost_build.btree_map(&["."]);
    prost_build
        .compile_protos(
            &["s2client-proto/s2clientprotocol/sc2api.proto"],
            &["s2client-proto"],
        )
        .unwrap();
}

fn main() {
    build_protos();
}
