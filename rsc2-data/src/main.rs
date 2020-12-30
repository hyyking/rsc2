use rsc2_data::from_file;

fn main() {
    let u = from_file("sc2-techtree/data/data_readable.json");
    dbg!(u);
}
