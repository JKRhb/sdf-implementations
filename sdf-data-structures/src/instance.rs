use std::{collections::HashMap, error::Error, fmt::Display};

use anyhow::Context;
use derive_builder::Builder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{traits::SdfDataStructure, util::none_extra};

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InfoBlock {
    #[builder(setter(into, strip_option), default)]
    pub title: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub modified: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub copyright: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub license: Option<String>,

    #[builder(setter(into))]
    pub message_id: String,
    #[builder(setter(into, strip_option), default)]
    pub previous_message_id: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub timestamp: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub features: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,

    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

// TODO: Move to utils

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommonQualities {
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub label: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_ref: Option<String>, // TODO: Add regex
    #[builder(setter(into, strip_option), default)]
    pub sdf_required: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdfMessage {
    #[builder(setter(strip_option))]
    pub info: Option<InfoBlock>,
    #[builder(setter(into, strip_option), default)]
    pub namespace: Option<HashMap<String, String>>,
    #[builder(setter(into, strip_option), default)]
    pub default_namespace: Option<String>,
    #[builder(setter(into, strip_option))]
    pub sdf_instance_of: SdfInstanceOf,
    pub sdf_instance: SdfInstance,
}

impl SdfDataStructure for SdfMessage {
    fn namespace(&self) -> Option<&HashMap<String, String>> {
        self.namespace.as_ref()
    }

    fn default_namespace(&self) -> Option<&String> {
        self.default_namespace.as_ref()
    }
}

#[derive(Debug)]
pub struct UrlResolutionError {
    pub error_message: String,
}

impl Error for UrlResolutionError {}

impl Display for UrlResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to resolved URI for prefix: {}.",
            self.error_message,
        )
    }
}

impl SdfMessage {
    fn generate_model_query_string(&self) -> Option<String> {
        let blah = [
            (&self.sdf_instance_of.lineage, "lineage"),
            (&self.sdf_instance_of.version, "version"),
            (&self.sdf_instance_of.min_version, "minVersion"),
            (&self.sdf_instance_of.max_version, "maxVersion"),
            (
                &self.sdf_instance_of.exclusive_min_version,
                "exclusiveMinVersion",
            ),
            (
                &self.sdf_instance_of.exclusive_max_version,
                "exclusiveMaxVersion",
            ),
        ];

        let query = blah
            .iter()
            .filter_map(|(value, key)| {
                if let Some(value) = value {
                    Some((value, key))
                } else {
                    None
                }
            })
            .map(|(value, key)| format!("{key}={}", value))
            .join("&");

        if query.is_empty() { None } else { Some(query) }
    }

    pub fn get_sdf_model_url(&self) -> anyhow::Result<Option<String>> {
        let target_namespace_url = self.get_target_namespace()?;

        if let Some(target_namespace_url) = target_namespace_url {
            if let Some(query_string) = self.generate_model_query_string() {
                Ok(Some(format!("{target_namespace_url}?{query_string}")))
            } else {
                Ok(Some(target_namespace_url))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_entry_point(&self) -> String {
        self.sdf_instance_of.entry_point.clone()
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdfInstanceOf {
    #[builder(setter(into))]
    pub entry_point: String,
    #[builder(setter(into, strip_option), default)]
    pub lineage: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub min_version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub max_version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub exclusive_min_version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub exclusive_max_version: Option<String>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdfInstance {
    #[builder(setter(into, strip_option), default)]
    pub thing_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_thing: Option<HashMap<String, SdfInstance>>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_object: Option<HashMap<String, SdfObject>>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_property: Option<HashMap<String, serde_json::Value>>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_context: Option<HashMap<String, serde_json::Value>>,
    #[builder(setter(strip_option), default)]
    pub sdf_action: Option<HashMap<String, Vec<ActionHistoryElement>>>,
    #[builder(setter(strip_option), default)]
    pub sdf_event: Option<HashMap<String, EventHistoryElement>>,
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "status")]
pub enum ActionState {
    Complete,
    Running,
    Error { error_message: String },
}

#[skip_serializing_none]
#[derive(Builder, PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistoryElement {
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    #[builder(setter(into))]
    pub timestamp: String,
    pub status: ActionState,
    #[builder(setter(into, strip_option), default)]
    pub input_value: Option<serde_json::Value>,
    #[builder(setter(into, strip_option), default)]
    pub output_value: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(Builder, PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventHistoryElement {
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    #[builder(setter(into))]
    pub timestamp: String,
    #[builder(setter(into, strip_option), default)]
    pub output_value: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdfObject {
    #[builder(setter(into, strip_option), default)]
    pub sdf_property: Option<HashMap<String, InteractionAffordance>>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_action: Option<HashMap<String, InteractionAffordance>>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_event: Option<HashMap<String, InteractionAffordance>>,

    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InteractionAffordance {
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub label: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub comment: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub sdf_ref: Option<String>, // TODO: Add regex
    #[builder(setter(into, strip_option), default)]
    pub sdf_required: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_sdf_instance() {
        let sdf_instance = SdfMessageBuilder::default()
            .namespace([("sensors".into(), "https://example.com/sensors".into())])
            .default_namespace("sensors")
            .info(
                InfoBlockBuilder::default()
                    .message_id("75532020-8f64-4daf-a241-fcb0b6dc4a44")
                    .build()
                    .unwrap(),
            )
            .sdf_instance_of(
                SdfInstanceOfBuilder::default()
                    .entry_point("#/sdfObject/envSensor")
                    .build()
                    .unwrap(),
            )
            .sdf_instance(
                SdfInstanceBuilder::default()
                    .sdf_context([("installationInfo".into(), json!({"mountType": "ceiling"}))])
                    .sdf_property([(
                        "status".to_string(),
                        serde_json::Value::String("operational".into()),
                    )])
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let serialized_sdf_instance = "{\"info\":{\"messageId\":\"75532020-8f64-4daf-a241-fcb0b6dc4a44\"},\"namespace\":{\"sensors\":\"https://example.com/sensors\"},\"defaultNamespace\":\"sensors\",\"sdfInstanceOf\":{\"entryPoint\":\"#/sdfObject/envSensor\"},\"sdfInstance\":{\"sdfProperty\":{\"status\":\"operational\"},\"sdfContext\":{\"installationInfo\":{\"mountType\":\"ceiling\"}}}}".to_string();

        assert_eq!(
            serde_json::to_string(&sdf_instance).unwrap(),
            serialized_sdf_instance
        );
    }
}
