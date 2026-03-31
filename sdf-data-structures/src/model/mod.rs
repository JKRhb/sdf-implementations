pub mod protocol_mappings;

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use derive_builder::Builder;
use json_merge_patch::json_merge_patch;
use json_pointer::JsonPointer;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{
    supplement::SdfSupplement,
    traits::{GlobalNameAggregator, GlobalNameContributor, SdfDataStructure},
    util::{default_bool_true, none_extra, skip_bool_true},
};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct InfoBlock {
    // TODO: Add modified and features
    #[builder(setter(into, strip_option), default)]
    pub title: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub copyright: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub license: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub lineage: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfModel {
    #[builder(setter(strip_option), default)]
    pub info: Option<InfoBlock>,
    #[builder(setter(into, strip_option), default)]
    pub namespace: Option<HashMap<String, String>>,
    #[builder(setter(into, strip_option), default)]
    pub default_namespace: Option<String>,
    #[builder(setter(strip_option), default)]
    pub sdf_thing: Option<HashMap<String, SdfThing>>,
    #[builder(setter(strip_option), default)]
    pub sdf_object: Option<HashMap<String, SdfObject>>,
    #[builder(setter(strip_option), default)]
    pub sdf_property: Option<HashMap<String, SdfProperty>>,
    #[builder(setter(strip_option), default)]
    pub sdf_action: Option<HashMap<String, SdfAction>>,
    #[builder(setter(strip_option), default)]
    pub sdf_event: Option<HashMap<String, SdfEvent>>,
    #[builder(setter(strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

impl SdfDataStructure for SdfModel {
    fn namespace(&self) -> Option<&HashMap<String, String>> {
        self.namespace.as_ref()
    }

    fn default_namespace(&self) -> Option<&String> {
        self.default_namespace.as_ref()
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
enum NewVersionType {
    Major = 3,
    Minor = 2,
    Patch = 1,
    Unchanged = 0,
}

impl SdfModel {
    /// Determines the set of global names of this SDF Model that conflict a list
    /// of existing SDF models.
    pub fn determine_global_name_collisions(
        &self,
        existing_sdf_models: Vec<&SdfModel>,
    ) -> HashSet<String> {
        let existing_global_names = existing_sdf_models.determine_global_names();
        let new_global_names = self.determine_global_names().unwrap_or_default();

        HashSet::from_iter(
            existing_global_names
                .intersection(&new_global_names)
                .cloned()
                .collect::<Vec<String>>(),
        )
    }

    /// Updates this SDF model using the amendments from the provided SDF supplement and returns the result.
    ///
    /// For the update to work, the version number of this model must adhere to semantic versioning.
    /// Depending on whether the changes applied to the model are backwards-compatible or not, or constitute
    /// a "fix", the new model will contain a version number that has been updated accordingly.
    ///
    /// The version bump will take into account the "most severe" change, i.e., if one change is
    /// non-backwards-compatible, it will cause the major version to be increased.
    pub fn update_sdf_model(&self, sdf_supplement: &SdfSupplement) -> anyhow::Result<SdfModel> {
        let mut serialized_model = serde_json::to_value(self)?;

        let current_version = self
            .get_version()
            .context("Model has no version defined!")?;

        let mut current_semantic_version = Version::parse(&current_version)
            .context("version quality does not adhere to semantic versioning!")?;

        let mut overall_new_version_type = NewVersionType::Unchanged;

        for amendment in &sdf_supplement.amend {
            for (key, value) in amendment.iter() {
                let delta = &value.delta;
                let fix = value.fix;

                let type_of_this_change: NewVersionType;

                if fix {
                    type_of_this_change = NewVersionType::Patch;
                } else {
                    let backwards_compatible_change = Self::check_for_backwards_compatibility(key);

                    if backwards_compatible_change {
                        type_of_this_change = NewVersionType::Minor;
                    } else {
                        type_of_this_change = NewVersionType::Major;
                    }
                }

                if type_of_this_change > overall_new_version_type {
                    overall_new_version_type = type_of_this_change;
                }

                let ptr = key.parse::<JsonPointer<_, _>>().unwrap();

                let target_definition = ptr.get_mut(&mut serialized_model).unwrap();

                json_merge_patch(target_definition, delta);
            }
        }

        match overall_new_version_type {
            NewVersionType::Major => current_semantic_version.major += 1,
            NewVersionType::Minor => current_semantic_version.minor += 1,
            NewVersionType::Patch => current_semantic_version.patch += 1,
            _ => {}
        }

        let new_sdf_model = serde_json::from_value::<Self>(serialized_model)?;

        let updated_model = new_sdf_model.update_version(current_semantic_version.to_string());

        Ok(updated_model)
    }

    fn check_for_backwards_compatibility(json_pointer: &String) -> bool {
        // TODO: Double-check whether this approach works
        let minor_change_keywords = vec![
            "#", // Top-level definitions
            "sdfThing",
            "sdfObject",
            "sdfProperty",
            "sdfAction",
            "sdfEvent",
            "sdfData",
            "label",
            "description",
            "$comment",
        ];

        if let Some(last_pointer_element) = &json_pointer.split("/").last() {
            minor_change_keywords.contains(last_pointer_element)
        } else {
            false
        }
    }

    pub fn get_default_namespace_url(&self) -> Option<String> {
        self.namespace
            .clone()?
            .get(&self.default_namespace.clone()?)
            .cloned()
    }

    pub fn get_version(&self) -> Option<String> {
        self.info.as_ref().and_then(|info| info.version.clone())
    }

    pub fn update_version(mut self, version: String) -> Self {
        let mut info = self.info.take().unwrap_or_default();

        info.version = Some(version);

        self.info = Some(info);

        self
    }

    pub fn get_lineage(&self) -> Option<String> {
        self.info.as_ref().and_then(|info| info.lineage.clone())
    }

    /// Determines the global names contributed by this `SdfModel`.
    ///
    /// Returns `None` if the `SdfModel` does not contribute any global names,
    /// i.e., when it does not define a target namespace via the `defaultNamespace`
    /// quality.
    pub fn determine_global_names(&self) -> Option<HashSet<String>> {
        let mut result = HashSet::new();

        let target_namespace_url = self.get_default_namespace_url()?;

        let prefix = format!("{target_namespace_url}#");

        if let Some(sdf_thing) = &self.sdf_thing {
            for (key, value) in sdf_thing {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        if let Some(sdf_object) = &self.sdf_object {
            for (key, value) in sdf_object {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        if let Some(sdf_property) = &self.sdf_property {
            for (key, value) in sdf_property {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        if let Some(sdf_action) = &self.sdf_action {
            for (key, value) in sdf_action {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        if let Some(sdf_event) = &self.sdf_event {
            for (key, value) in sdf_event {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        if let Some(sdf_data) = &self.sdf_data {
            for (key, value) in sdf_data.iter() {
                value.get_global_name(&prefix, &mut result, key);
            }
        }

        Some(result)
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfThing {
    #[builder(setter(strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub sdf_thing: Option<HashMap<String, SdfThing>>,
    #[builder(setter(strip_option), default)]
    pub sdf_object: Option<HashMap<String, SdfObject>>,
    #[builder(setter(strip_option), default)]
    pub sdf_property: Option<HashMap<String, SdfProperty>>,
    #[builder(setter(strip_option), default)]
    pub sdf_action: Option<HashMap<String, SdfAction>>,
    #[builder(setter(strip_option), default)]
    pub sdf_event: Option<HashMap<String, SdfEvent>>,
    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,

    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

impl GlobalNameContributor for SdfThing {
    const QUALITY_NAME: &'static str = "sdfThing";

    fn get_global_name(&self, prefix: &String, result: &mut HashSet<String>, given_name: &String) {
        let global_name = format!("{prefix}/{}/{given_name}", Self::QUALITY_NAME);
        result.insert(global_name.clone());

        if let Some(sdf_thing) = &self.sdf_thing {
            for (key, value) in sdf_thing.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_object) = &self.sdf_object {
            for (key, value) in sdf_object.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_action) = &self.sdf_action {
            for (key, value) in sdf_action.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_property) = &self.sdf_property {
            for (key, value) in sdf_property.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_event) = &self.sdf_event {
            for (key, value) in sdf_event.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_data) = &self.sdf_data {
            for (key, value) in sdf_data.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfObject {
    #[builder(setter(strip_option), default)]
    pub sdf_property: Option<HashMap<String, SdfProperty>>,
    #[builder(setter(strip_option), default)]
    pub sdf_action: Option<HashMap<String, SdfAction>>,
    #[builder(setter(strip_option), default)]
    pub sdf_event: Option<HashMap<String, SdfEvent>>,
    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,

    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(setter(strip_option), default)]
    pub min_items: Option<u64>,
    #[builder(setter(strip_option), default)]
    pub max_items: Option<u64>,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

impl GlobalNameContributor for SdfObject {
    const QUALITY_NAME: &'static str = "sdfObject";

    fn get_global_name(&self, prefix: &String, result: &mut HashSet<String>, given_name: &String) {
        let global_name = format!("{prefix}/{}/{given_name}", Self::QUALITY_NAME);
        result.insert(global_name.clone());

        if let Some(sdf_action) = &self.sdf_action {
            for (key, value) in sdf_action.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_property) = &self.sdf_property {
            for (key, value) in sdf_property.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_event) = &self.sdf_event {
            for (key, value) in sdf_event.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }

        if let Some(sdf_data) = &self.sdf_data {
            for (key, value) in sdf_data.iter() {
                value.get_global_name(&global_name, result, key);
            }
        }
    }
}

impl GlobalNameAggregator for Vec<&SdfModel> {
    fn determine_global_names(&self) -> HashSet<String> {
        let mut result = HashSet::new();

        for sdf_model in self {
            let global_names = sdf_model.determine_global_names().unwrap_or_default();

            for global_name in global_names.iter() {
                result.insert(global_name.clone());
            }
        }

        result
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfData {
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub r#type: Option<SchemaDefinition>,

    #[builder(setter(into, strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub sdf_choice: Option<HashMap<String, SdfData>>,
    #[builder(setter(strip_option), default)]
    pub r#enum: Option<Vec<String>>,

    #[builder(setter(strip_option), default)]
    pub r#const: Option<serde_json::Value>,
    #[builder(setter(strip_option), default)]
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
    #[serde(flatten, deserialize_with = "deserialize_additional_sdf_data")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

pub fn deserialize_additional_sdf_data<'de, D>(
    deserializer: D,
) -> Result<Option<Map<String, Value>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let mut deserialized_map = Map::deserialize(deserializer)?;
    deserialized_map.retain(|key, _| key != "type");
    Ok((!deserialized_map.is_empty()).then_some(deserialized_map))
}

impl GlobalNameContributor for SdfData {
    const QUALITY_NAME: &'static str = "sdfData";
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SchemaDefinition {
    Boolean,
    String(StringSchema),
    Integer(NumericSchema<i64>),
    Number(NumericSchema<f64>),
    Array(ArraySchema),
    Object(ObjectSchema),
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Builder)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct StringSchema {
    #[builder(setter(strip_option), default)]
    pub min_length: Option<u64>,
    #[builder(setter(strip_option), default)]
    pub max_length: Option<u64>,
    #[builder(setter(into, strip_option), default)]
    pub pattern: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub format: Option<String>,
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Builder)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct NumericSchema<T> {
    #[builder(setter(strip_option), default)]
    pub minimum: Option<T>,
    #[builder(setter(strip_option), default)]
    pub maximum: Option<T>,
    #[builder(setter(strip_option), default)]
    pub exclusive_minimum: Option<T>,
    #[builder(setter(strip_option), default)]
    pub exclusive_maximum: Option<T>,
    #[builder(setter(strip_option), default)]
    pub multiple_of: Option<T>,
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Builder)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ArraySchema {
    #[builder(setter(strip_option), default)]
    pub min_items: Option<u64>,
    #[builder(setter(strip_option), default)]
    pub max_items: Option<u64>,
    #[builder(setter(strip_option), default)]
    pub unique_items: Option<bool>,
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Builder)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema {
    #[builder(setter(into, strip_option), default)]
    pub required: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub properties: Option<HashMap<String, SdfData>>,
}

// #[skip_serializing_none]
// #[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
// pub struct PropertyProtocolMap {
//     pub coap: Option<PropertyCoapProtocolMap>,
//     pub http: Option<PropertyCoapProtocolMap>,
// }

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfProperty {
    #[serde(flatten)]
    #[builder(default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub internal_data: SdfData,

    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub readable: bool,
    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub writable: bool,
    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub observable: bool,
    // pub sdf_protocol_map: Option<PropertyProtocolMap>,
}

impl GlobalNameContributor for SdfProperty {
    const QUALITY_NAME: &'static str = "sdfProperty";
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfAction {
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[builder(setter(strip_option), default)]
    pub sdf_input_data: Option<SdfData>,
    #[builder(setter(strip_option), default)]
    pub sdf_output_data: Option<SdfData>,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

impl GlobalNameContributor for SdfAction {
    const QUALITY_NAME: &'static str = "sdfAction";
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SdfEvent {
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[builder(setter(strip_option), default)]
    pub sdf_output_data: Option<SdfData>,
    #[serde(flatten)]
    #[builder(setter(into), default)]
    pub additional_qualities: HashMap<String, Value>,
}

impl GlobalNameContributor for SdfEvent {
    const QUALITY_NAME: &'static str = "sdfEvent";
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_common_qualities() {
        let common_qualities = CommonQualitiesBuilder::default()
            .comment("This is a comment")
            .build()
            .unwrap();

        let serialized_common_qualities = "{\"$comment\":\"This is a comment\"}".to_string();

        assert_eq!(
            serde_json::to_string(&common_qualities).unwrap(),
            serialized_common_qualities
        );
    }

    #[test]
    fn test_sdf_property() {
        let sdf_property = SdfPropertyBuilder::default()
            .writable(false)
            .build()
            .unwrap();

        let serialized_sdf_property = "{\"writable\":false}".to_string();

        assert_eq!(
            serde_json::to_string(&sdf_property).unwrap(),
            serialized_sdf_property
        );
    }

    #[test]
    fn test_const_and_default() {
        let sdf_data = SdfDataBuilder::default()
            .r#const(serde_json::Value::Null)
            .default_value(json!(5))
            .build()
            .unwrap();

        let serialized_sdf_property = "{\"const\":null,\"default\":5}".to_string();

        assert_eq!(
            serde_json::to_string(&sdf_data).unwrap(),
            serialized_sdf_property
        );
    }

    #[test]
    fn test_rfc_9880_example() {
        let value: Value = json!(
          {
            "info": {
              "title": "Example document for SDF (Semantic Definition Format)",
              "version": "2019-04-24",
              "copyright": "Copyright 2019 Example Corp. All rights reserved.",
              "license": "https://example.com/license"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "Switch": {
                "sdfProperty": {
                  "value": {
                    "description": "The state of the switch; false for off and true for on.",
                    "type": "boolean"
                  }
                },
                "sdfAction": {
                  "on": {
                    "description": "Turn the switch on; equivalent to setting value to true."
                  },
                  "off": {
                    "description": "Turn the switch off; equivalent to setting value to false."
                  },
                  "toggle": {
                    "description": "Toggle the switch; equivalent to setting value to its complement."
                  }
                }
              }
            }
          }
        );

        let sdf_model =
            serde_json::from_value::<SdfModel>(value).expect("Error deserializing SDF model");

        let global_names = sdf_model
            .determine_global_names()
            .expect("SDF Model does not contribute global names.");

        let expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfObject/Switch".to_string(),
            "https://example.com/capability/cap#/sdfObject/Switch/sdfProperty/value".to_string(),
            "https://example.com/capability/cap#/sdfObject/Switch/sdfAction/on".to_string(),
            "https://example.com/capability/cap#/sdfObject/Switch/sdfAction/off".to_string(),
            "https://example.com/capability/cap#/sdfObject/Switch/sdfAction/toggle".to_string(),
        ]);

        assert_eq!(global_names, expected_result);
    }

    #[test]
    fn test_nested_sdf_model() {
        let value = json!(
          {
            "info": {
              "title": "Example document for SDF (Semantic Definition Format)",
              "version": "2019-04-24",
              "copyright": "Copyright 2019 Example Corp. All rights reserved.",
              "license": "https://example.com/license"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfThing": {
              "foo": {
                "sdfThing": {
                  "bar": {
                    "sdfObject": {
                      "baz": {
                        "sdfAction": {
                          "testAction": {
                            "title": "This is a test."
                          }
                        },
                        "sdfData": {
                          "greatData": {
                            "description": "This is great data!"
                          }
                        }
                      }
                    }
                  }
                },
                "sdfProperty": {
                  "testProperty": {
                    "type": "string"
                  }
                }
              }
            },
            "sdfEvent": {
              "topLevelSdfEvent": {
                "description": "This is an amazing event affordance."
              }
            },
            "sdfData": {
              "evenBetterData": {
                "description": "This is even better data!"
              }
            }
          }
        );

        let sdf_model =
            serde_json::from_value::<SdfModel>(value).expect("Error deserializing SDF model");

        let global_names = sdf_model
            .determine_global_names()
            .expect("SDF Model does not contribute global names.");

        let expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfThing/foo".to_string(),
            "https://example.com/capability/cap#/sdfThing/foo/sdfThing/bar".to_string(),
            "https://example.com/capability/cap#/sdfThing/foo/sdfThing/bar/sdfObject/baz/sdfAction/testAction".to_string(),
            "https://example.com/capability/cap#/sdfThing/foo/sdfThing/bar/sdfObject/baz/sdfData/greatData".to_string(),
            "https://example.com/capability/cap#/sdfThing/foo/sdfThing/bar/sdfObject/baz".to_string(),
            "https://example.com/capability/cap#/sdfThing/foo/sdfProperty/testProperty".to_string(),
            "https://example.com/capability/cap#/sdfEvent/topLevelSdfEvent".to_string(),
            "https://example.com/capability/cap#/sdfData/evenBetterData".to_string(),
        ]);

        assert_eq!(global_names, expected_result);
    }

    #[test]
    fn test_unions_of_global_names() {
        let value1 = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "foo": {
                "sdfProperty": {
                  "bar": {}
                }
              }
            }
          }
        );

        let value2 = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "foo": {
                "sdfProperty": {
                  "bar": {},
                  "baz": {}
                }
              }
            }
          }
        );

        let sdf_model1 = serde_json::from_value::<SdfModel>(value1)
            .expect("Deserialization of SDF Model 1 failed!");
        let sdf_model2 = serde_json::from_value::<SdfModel>(value2)
            .expect("Deserialization of SDF Model 2 failed!");

        let global_names = vec![&sdf_model1, &sdf_model2].determine_global_names();

        let expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfObject/foo".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo/sdfProperty/bar".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo/sdfProperty/baz".to_string(),
        ]);

        assert_eq!(global_names, expected_result);
    }
}
