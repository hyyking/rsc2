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

fn build_ids() -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;

    use rsc2_data::{enumeration, from_file};

    let repr = from_file("../rsc2-data/sc2-techtree/data/data.json").unwrap();
    let out_file = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ids.rs");
    let f = &mut OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_file)
        .unwrap();

    writeln!(f, "{}", enumeration::from_slice("AbilityId", &repr.ability))?;
    writeln!(f, "{}", enumeration::from_slice("UnitId", &repr.unit))?;
    writeln!(f, "{}", enumeration::from_slice("UpgradeId", &repr.upgrade))?;
    Ok(())
}

fn main() {
    build_protos();
    build_ids().unwrap();
}
