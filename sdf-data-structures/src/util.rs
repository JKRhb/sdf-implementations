use serde::Deserialize;
use serde_json::{Map, Value};

pub fn none_extra<'de, D>(deserializer: D) -> Result<Option<Map<String, Value>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Map::deserialize(deserializer)?;
    Ok((!s.is_empty()).then_some(s))
}

/// Helper function to return a boolean value as a default during serialization.
#[inline]
pub(crate) fn default_bool_true() -> bool {
    true
}

/// Helper function to dereference a boolean value during serialization.
#[inline]
pub(crate) fn skip_bool_true(value: &bool) -> bool {
    *value
}
