use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    pub name: String,
    pub values: u32,
}

pub fn encode(properties: &[Property], values: &[u32]) -> u32 {
    let mut data = 0u32;
    let mut multiplier = 1u32;
    for (p, v) in properties.iter().zip(values) {
        data += v * multiplier;
        multiplier *= p.values;
    }
    data
}

pub fn decode(properties: &[Property], mut data: u32) -> Vec<u32> {
    properties
        .iter()
        .map(|p| {
            let v = data % p.values;
            data /= p.values;
            v
        })
        .collect()
}

pub fn state_count(properties: &[Property]) -> u32 {
    properties.iter().map(|p| p.values).product()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ValueExpr {
    Literal(u32),
    Property { property: String },
}

pub fn resolve(expr: &ValueExpr, properties: &[Property], values: &[u32]) -> u32 {
    match expr {
        ValueExpr::Literal(v) => *v,
        ValueExpr::Property { property } => {
            let idx = properties
                .iter()
                .position(|p| &p.name == property)
                .unwrap_or_else(|| panic!("shape references unknown property '{property}'"));
            values[idx]
        }
    }
}

pub fn matches_when(when: &HashMap<String, u32>, properties: &[Property], values: &[u32]) -> bool {
    when.iter().all(|(name, expected)| {
        let idx = properties
            .iter()
            .position(|p| &p.name == name)
            .unwrap_or_else(|| panic!("'when' references unknown property '{name}'"));
        values[idx] == *expected
    })
}
