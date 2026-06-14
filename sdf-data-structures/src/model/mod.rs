// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

pub mod affordances;
pub mod common_qualities;
pub mod info_block;
pub mod protocol_mappings;
pub mod schema_definition;
pub mod sdf_context;
pub mod sdf_data;
pub mod sdf_object;
pub mod sdf_thing;

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use derive_builder::Builder;
use json_merge_patch::json_merge_patch;
use json_pointer::JsonPointer;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{
    model::{
        affordances::{sdf_action::SdfAction, sdf_event::SdfEvent},
        info_block::InfoBlock,
        sdf_context::SdfContext,
        sdf_data::SdfData,
        sdf_object::SdfObject,
        sdf_thing::SdfThing,
    },
    supplement::SdfSupplement,
    traits::{GlobalNameAggregator, GlobalNameContributor, SdfDataStructure},
    util::none_extra,
};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

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
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
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
    pub sdf_property: Option<HashMap<String, SdfAction>>,
    #[builder(setter(strip_option), default)]
    pub sdf_action: Option<HashMap<String, SdfAction>>,
    #[builder(setter(strip_option), default)]
    pub sdf_event: Option<HashMap<String, SdfEvent>>,
    #[builder(setter(strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<HashMap<String, Value>>,
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
    pub fn list_config_parameters(
        &self,
        _entry_point: &str,
    ) -> anyhow::Result<HashMap<String, SdfContext>> {
        let mut result = HashMap::<String, SdfContext>::new();

        // TODO: Refactor and nest
        if let Some(sdf_thing_map) = &self.sdf_thing {
            for (thing_key, sdf_thing) in sdf_thing_map.iter() {
                if let Some(sdf_context_definitions) = &sdf_thing.sdf_context {
                    for (context_key, value) in sdf_context_definitions.iter() {
                        let path = ["sdfThing", thing_key, "sdfContext", context_key].join("/");
                        result.insert(path, value.clone());
                    }
                }
            }
        }

        if let Some(sdf_object_map) = &self.sdf_object {
            for (thing_key, sdf_thing) in sdf_object_map.iter() {
                if let Some(sdf_context_definitions) = &sdf_thing.sdf_context {
                    for (context_key, value) in sdf_context_definitions.iter() {
                        let path = ["sdfThing", thing_key, "sdfContext", context_key].join("/");
                        result.insert(path, value.clone());
                    }
                }
            }
        }

        Ok(result)
    }

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

    fn check_for_backwards_compatibility(json_pointer: &str) -> bool {
        // TODO: Double-check whether this approach works
        let minor_change_keywords = vec![
            "#", // Top-level definitions
            "sdfThing",
            "sdfObject",
            "sdfProperty",
            "sdfAction",
            "sdfEvent",
            "sdfContext",
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

    /// Returns the default namespace URL from the `namespace` quality as indicated
    /// by the value of the `defaultNamespace` quality.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::model::SdfModelBuilder;
    /// use std::collections::HashMap;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfModelBuilder::default()
    ///     .namespace(HashMap::from_iter(vec![("foo".to_string(), "https://example.org".to_string())]))
    ///     .default_namespace("foo")
    ///     .build()?;
    ///
    /// assert_eq!(model.get_default_namespace_url(), Some("https://example.org".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_default_namespace_url(&self) -> Option<String> {
        self.namespace
            .clone()?
            .get(&self.default_namespace.clone()?)
            .cloned()
    }

    /// Returns the value of the `version` quality within this model's `info` block, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::model::SdfModelBuilder;
    /// use sdf_data_structures::model::info_block::InfoBlockBuilder;
    /// #
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfModelBuilder::default()
    ///     .info(InfoBlockBuilder::default().version("1.0.0").build()?)
    ///     .build()?;
    ///
    /// assert_eq!(model.get_version(), Some("1.0.0".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_version(&self) -> Option<String> {
        self.info.as_ref().and_then(|info| info.version.clone())
    }

    /// Updates the value of the `version` quality within this model's `info` block and
    /// returns the updated model.
    ///
    /// If no `info` block is defined, it will be created by this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::model::SdfModel;
    ///
    /// let mut model = SdfModel::default();
    ///
    /// model = model.update_version("1.0.0".to_string());
    ///
    /// assert_eq!(model.get_version(), Some("1.0.0".to_string()));
    /// ```
    pub fn update_version(mut self, version: String) -> Self {
        let mut info = self.info.take().unwrap_or_default();

        info.version = Some(version);

        self.info = Some(info);

        self
    }

    /// Returns the value of the `lineage` quality within this model's `info` block, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::model::SdfModelBuilder;
    /// use sdf_data_structures::model::info_block::InfoBlockBuilder;
    /// #
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfModelBuilder::default()
    ///     .info(InfoBlockBuilder::default().lineage("foobar").build()?)
    ///     .build()?;
    ///
    /// assert_eq!(model.get_lineage(), Some("foobar".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

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
