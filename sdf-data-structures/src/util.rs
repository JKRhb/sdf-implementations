use serde::Deserialize;
use serde_json::{Map, Value};

pub fn none_extra<'de, D>(deserializer: D) -> Result<Option<Map<String, Value>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Map::deserialize(deserializer)?;
    Ok((!s.is_empty()).then_some(s))
}
