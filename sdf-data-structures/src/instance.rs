// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, error::Error, fmt::Display};

use derive_builder::Builder;
use itertools::Itertools;
use ploidy_pointer::{JsonPointee, JsonPointeeExt, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{model::SdfModel, traits::SdfDataStructure, util::none_extra};

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
    pub additional_qualities: Option<HashMap<String, Value>>,
}

// TODO: Move to utils

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
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
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
            .filter_map(|(value, key)| value.as_ref().map(|value| (value, key)))
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

    pub fn get_entry_point(&self) -> &str {
        &self.sdf_instance_of.entry_point
    }

    pub fn resolve_model_pointer(&self, relative_pointer: &str) -> String {
        let trimmed_entry_pointer = self
            .get_entry_point()
            .trim_end_matches('/')
            .trim_start_matches("#/");

        let trimmed_relative_pointer = relative_pointer.trim_start_matches('/');

        [trimmed_entry_pointer, trimmed_relative_pointer].join("/")
    }

    pub fn resolve_pointer_against_instance<'a, T: Clone + JsonPointerTarget<'a>>(
        &'a self,
        relative_pointer: &str,
    ) -> anyhow::Result<T> {
        // TODO: Decide where to put this
        let resolved_relative_pointer = relative_pointer.trim_start_matches(self.get_entry_point());

        let prefixed_relative_pointer = "/sdfInstance".to_string() + resolved_relative_pointer;

        let result = self.pointer::<T>(&prefixed_relative_pointer)?;

        Ok(result)
    }

    // TODO: Move to SDF model
    pub fn resolve_absolute_pointer_against_model<
        'a,
        T: std::fmt::Debug + Clone + JsonPointerTarget<'a>,
    >(
        &self,
        absolute_pointer: &str,
        sdf_model: &'a SdfModel,
    ) -> anyhow::Result<T> {
        let resolved_pointer = absolute_pointer.trim_start_matches('#');

        let result = sdf_model.pointer::<T>(resolved_pointer)?;

        Ok(result)
    }

    pub fn resolve_pointer_against_model<'a, T: std::fmt::Debug + Clone + JsonPointerTarget<'a>>(
        &self,
        relative_pointer: &str,
        sdf_model: &'a SdfModel,
    ) -> anyhow::Result<T> {
        let resolved_pointer = self.get_entry_point().to_owned() + relative_pointer;
        let resolved_pointer = resolved_pointer.trim_start_matches('#');

        let result = sdf_model.pointer::<T>(resolved_pointer)?;

        Ok(result)
    }
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonPointee, JsonPointerTarget)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
#[serde(tag = "status")]
pub enum ActionState {
    Complete,
    Running,
    Error { error_message: String },
}

#[skip_serializing_none]
#[derive(
    Builder, PartialEq, Serialize, Deserialize, Debug, Clone, JsonPointee, JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
#[derive(
    Builder, PartialEq, Serialize, Deserialize, Debug, Clone, JsonPointee, JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
