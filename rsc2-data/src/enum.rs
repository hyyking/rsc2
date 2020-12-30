use std::fmt;

pub struct Enum<'a> {
    name: &'static str,
    members: Vec<EnumMember<'a>>,
}
pub struct EnumMember<'a> {
    name: &'a str,
    value: &'a usize,
}

impl<'a> fmt::Display for Enum<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DERIVES: &str = "#[derive(Debug, Copy, Clone, Eq, PartialEq)]\n";
        write!(f, "{}", DERIVES)?;
        write!(f, "pub enum {} {{\n", self.name)?;
        for member in &self.members {
            write!(f, "    {},\n", member)?;
        }
        write!(f, "{}", "}\n")
    }
}
impl<'a> fmt::Display for EnumMember<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = to_camelcase(self.name);
        write!(f, "{} = {}", name, self.value)
    }
}

fn to_camelcase(input: &str) -> String {
    let mut output = String::with_capacity(64);
    for part in input.split("_") {
        if part.len() > 0 {
            output.push_str(&part.get(..1).unwrap().to_uppercase());
        }
        if part.len() > 1 {
            output.push_str(&part.get(1..).unwrap().to_lowercase())
        }
    }
    output
}

pub fn from_slice<'a, D: crate::ToEnum>(name: &'static str, data: &'a [D]) -> Enum<'a> {
    let members: Vec<_> = data
        .iter()
        .map(|d| EnumMember {
            name: d.name(),
            value: d.value(),
        })
        .collect();
    Enum { name, members }
}
