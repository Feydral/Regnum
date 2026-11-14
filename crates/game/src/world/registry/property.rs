use std::collections::HashMap;

use serde::Deserialize;

pub const MAX_STATES: u32 = 1 << 16;

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    pub name: String,
    pub values: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ValueExpr {
    Literal(u32),
    FromProperty { property: String },
}

impl ValueExpr {
    pub fn resolve(&self, properties: &[Property], values: &[u32]) -> Result<u32, String> {
        match self {
            ValueExpr::Literal(value) => Ok(*value),
            ValueExpr::FromProperty { property } => properties
                .iter()
                .position(|p| &p.name == property)
                .map(|i| values[i])
                .ok_or_else(|| format!("unknown property '{property}'")),
        }
    }
}

pub fn state_count(properties: &[Property]) -> Option<u32> {
    properties
        .iter()
        .try_fold(1u32, |count, p| count.checked_mul(p.values))
}

pub fn decode(properties: &[Property], mut data: u32) -> Vec<u32> {
    properties
        .iter()
        .map(|p| {
            let value = data % p.values;
            data /= p.values;
            value
        })
        .collect()
}

pub fn encode(properties: &[Property], values: &[u32]) -> u32 {
    let mut data = 0;
    let mut stride = 1;
    for (property, value) in properties.iter().zip(values) {
        data += value * stride;
        stride *= property.values;
    }
    data
}

pub fn matches_when(when: &HashMap<String, u32>, properties: &[Property], values: &[u32]) -> bool {
    when.iter().all(|(name, expected)| {
        properties
            .iter()
            .position(|p| &p.name == name)
            .is_some_and(|i| values[i] == *expected)
    })
}

pub fn validate_properties(properties: &[Property]) -> Result<u32, String> {
    for (i, property) in properties.iter().enumerate() {
        if property.values == 0 {
            return Err(format!("property '{}' has no values", property.name));
        }
        if properties[..i].iter().any(|p| p.name == property.name) {
            return Err(format!("duplicate property '{}'", property.name));
        }
    }
    match state_count(properties) {
        Some(count) if count <= MAX_STATES => Ok(count),
        _ => Err(format!("more than {MAX_STATES} states")),
    }
}

pub fn validate_when(when: &HashMap<String, u32>, properties: &[Property]) -> Result<(), String> {
    for (name, value) in when {
        let Some(property) = properties.iter().find(|p| &p.name == name) else {
            return Err(format!("'when' uses unknown property '{name}'"));
        };
        if *value >= property.values {
            return Err(format!(
                "'when' value {value} is out of range for property '{name}'"
            ));
        }
    }
    Ok(())
}
