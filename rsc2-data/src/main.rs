use rsc2_data::{enumeration, from_file};

fn main() {
    let repr = from_file("sc2-techtree/data/data_readable.json").unwrap();

    // println!("{}", enumeration::from_slice("AbilityId", &repr.ability));
    // println!("{}", enumeration::from_slice("UnitId", &repr.unit));
    // println!("{}", enumeration::from_slice("UpgradeId", &repr.upgrade));
}
