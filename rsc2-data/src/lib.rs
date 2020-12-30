use std::error::Error;

pub mod raw;
pub use raw::RawData;

#[doc(hidden)]
pub mod r#enum;
pub use r#enum as enumeration;

pub trait ToEnum {
    fn name(&self) -> &str;
    fn value(&self) -> &usize;
}

pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<RawData, Box<dyn Error>> {
    let r = std::io::BufReader::new(std::fs::File::open(path)?);
    let data = serde_json::from_reader(r)?;
    Ok(data)
}
