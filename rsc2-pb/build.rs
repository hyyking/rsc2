const SC2PB: &str = "https://github.com/Blizzard/s2client-proto";

const DERIVE_WRAPENUM: &str = "#[derive(::rsc2_macro::WrapEnum)]";
const DERIVE_TRYINTOENUM: &str = "#[derive(::rsc2_macro::TryIntoEnum)]";

fn main() {
    // Download the repository
    match git2::Repository::open("./s2client-proto") {
        Ok(_) => {}
        Err(e_open) => match git2::Repository::clone(SC2PB, "./s2client-proto") {
            Ok(_) => (),
            Err(e_clone) => panic!("failed to open: {} and clone {}", e_open, e_clone),
        },
    }

    let mut prost_build = prost_build::Config::new();
    prost_build.btree_map(&["."]);

    prost_build.type_attribute("Request.request", DERIVE_WRAPENUM);
    prost_build.type_attribute("Status", DERIVE_TRYINTOENUM);

    prost_build
        .compile_protos(
            &["s2client-proto/s2clientprotocol/sc2api.proto"],
            &["s2client-proto"],
        )
        .unwrap();
}
