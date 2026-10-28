//! A fixture for the terminals' shots: Rust, as bat shows it.

use std::collections::BTreeMap;
use std::fmt;

/// A colour role and the matugen colour it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub name: String,
    pub source: Option<String>,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(from) => write!(f, "{} <- {from}", self.name),
            None => write!(f, "{} (ours)", self.name),
        }
    }
}

fn main() {
    let roles: BTreeMap<&str, u8> =
        [("primary", 0xff), ("cold", 0x85)].into();
    for (name, value) in &roles {
        println!("{name}: {value:#04x}");
    }
}
