// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use semver::Version;
use serde_json::{Map, Value};

static MODEL_ID_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(serde::Serialize, Debug, Clone)]
pub struct SdfModelEntry {
    id: String,
    pub model: serde_json::Map<String, serde_json::Value>,
    pub version: String,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl PartialOrd for SdfModelEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl PartialEq for SdfModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.namespace == other.namespace
            && self.lineage == other.lineage
    }
}

impl SdfModelEntry {
    pub fn new(
        model: serde_json::Map<String, serde_json::Value>,
        version: String,
        namespace: String,
        lineage: Option<String>,
    ) -> SdfModelEntry {
        SdfModelEntry {
            id: Self::get_next_model_id(),
            model,
            lineage,
            namespace,
            version,
        }
    }

    fn get_next_model_id() -> String {
        MODEL_ID_SEQ.fetch_add(1, Ordering::SeqCst);
        MODEL_ID_SEQ.load(Ordering::SeqCst).to_string()
    }
}

/// Determines the set of global names from the `new_sdf_model` that conflict with
/// `existing_sdf_models` from a different lineage.
///
/// Note that an undefined lineage also counts as a lineage itself.
pub(crate) fn determine_global_name_collisions(
    new_sdf_model: &Map<String, Value>,
    existing_sdf_models: Vec<&Map<String, Value>>,
) -> HashSet<String> {
    // TODO: Put into its own function
    let new_model_lineage = get_info_block_field_value(new_sdf_model, "lineage");

    let models_from_different_lineages = existing_sdf_models
        .into_iter()
        .filter(|existing_sdf_model| {
            let existing_model_lineage = get_info_block_field_value(existing_sdf_model, "lineage");

            // TODO: Revisit this
            new_model_lineage != existing_model_lineage
        })
        .collect::<Vec<_>>();

    let existing_global_names = determine_global_names_for_models(models_from_different_lineages);
    let new_global_names = determine_global_names_for_model(new_sdf_model);

    HashSet::from_iter(
        existing_global_names
            .intersection(&new_global_names)
            .cloned()
            .collect::<Vec<String>>(),
    )
}

pub(crate) fn get_info_block_field_value(
    sdf_document: &Map<String, Value>,
    field_name: &str,
) -> Option<String> {
    sdf_document
        .get("info")
        .and_then(|info_block| info_block.get(field_name))
        .and_then(|lineage_value| lineage_value.as_str())
        .map(|lineage_value| lineage_value.to_string())
}

pub(crate) fn add_model_to_state(
    models: &mut Vec<SdfModelEntry>,
    new_sdf_model: Map<String, Value>,
) -> actix_web::Result<()> {
    let existing_sdf_models = models
        .iter()
        .map(|sdf_model_entry| &sdf_model_entry.model)
        .collect::<Vec<_>>();

    let lineage_exists = check_for_existing_lineage(&new_sdf_model, existing_sdf_models.clone())?;

    if lineage_exists {
        return Err(actix_web::error::ErrorBadRequest("Lineage already exists!"));
    }

    let collisions = determine_global_name_collisions(&new_sdf_model, existing_sdf_models);

    let namespace = obtain_namespace_url(&new_sdf_model)?
        .ok_or(actix_web::error::ErrorBadRequest("Missing namespace URL!"))?;

    let lineage = get_info_block_field_value(&new_sdf_model, "lineage");
    let version = get_info_block_field_value(&new_sdf_model, "version")
        .ok_or(actix_web::error::ErrorBadRequest("Missing version!"))?;

    if collisions.is_empty() {
        models.push(SdfModelEntry::new(
            new_sdf_model.clone(),
            version,
            namespace,
            lineage,
        ));
        return Ok(());
    }

    Err(actix_web::error::ErrorBadRequest(
        "Definition collisions detected!",
    ))
}

pub(crate) fn check_for_existing_lineage(
    new_sdf_model: &Map<String, Value>,
    existing_sdf_models: Vec<&Map<String, Value>>,
) -> actix_web::Result<bool> {
    let target_namespace_url = obtain_namespace_url(new_sdf_model)?;
    let lineage = get_info_block_field_value(new_sdf_model, "lineage");

    for existing_sdf_model in existing_sdf_models {
        let existing_target_namespace_url = obtain_namespace_url(existing_sdf_model)?;
        let existing_lineage = get_info_block_field_value(existing_sdf_model, "lineage");

        if target_namespace_url == existing_target_namespace_url && lineage == existing_lineage {
            return Ok(true);
        }
    }

    Ok(false)
}

pub(crate) fn obtain_namespace_url(
    supplement: &serde_json::Map<String, serde_json::Value>,
) -> actix_web::Result<Option<String>> {
    // TODO: Consider factoring out parts of this code to be able to reuse it
    let namespace_option = supplement.get("namespace");
    let default_namespace_option = supplement.get("defaultNamespace");

    if namespace_option.is_none() || default_namespace_option.is_none() {
        return Ok(None);
    }

    if namespace_option.is_none() && default_namespace_option.is_some() {
        return Err(actix_web::error::ErrorBadRequest(
            "The presence of a default namespace requires a namespace map",
        ));
    }

    let default_namespace_string =
        default_namespace_option
            .unwrap()
            .as_str()
            .ok_or(actix_web::error::ErrorBadRequest(
                "Wrong type for default namespace, expected string",
            ))?;

    let namespace_map =
        namespace_option
            .unwrap()
            .as_object()
            .ok_or(actix_web::error::ErrorBadRequest(
                "Wrong type for default namespace, expected Map",
            ))?;

    namespace_map
        .get(default_namespace_string)
        .ok_or(actix_web::error::ErrorBadRequest(
            "Default namespace is not included in namespace map",
        ))?
        .as_str()
        .ok_or(actix_web::error::ErrorBadRequest(
            "Value of default namespace in namespace map is not a string.",
        ))
        .map(|x| Some(x.to_string()))
}

fn determine_global_names_for_affordance_or_data_quality(
    path_elements: Vec<&str>,
    result: &mut HashSet<String>,
    parent_definition: &Map<String, Value>,
    prefix: &str,
) {
    if let Some(current_definition) = parent_definition.get(prefix) {
        let mut path_elements = path_elements.clone();
        path_elements.push(prefix);

        if let Some(object) = current_definition.as_object() {
            for (key, _) in object {
                let mut path_elements = path_elements.clone();
                path_elements.push(key);

                result.insert(path_elements.join("/"));
            }
        }
    }
}

fn determine_global_names_for_sdf_grouping(
    path_elements: Vec<&str>,
    result: &mut HashSet<String>,
    parent_definition: &Map<String, Value>,
    prefix: &str,
) {
    if let Some(current_definition) = parent_definition.get(prefix) {
        let mut path_elements = path_elements.clone();
        path_elements.push(prefix);

        if let Some(object) = current_definition.as_object() {
            for (key, value) in object {
                let mut path_elements = path_elements.clone();
                path_elements.push(key);

                result.insert(path_elements.join("/"));

                determine_global_names_for_affordance_or_data_qualities(
                    path_elements.clone(),
                    result,
                    value.as_object().unwrap(),
                );

                if prefix == "sdfThing" {
                    determine_global_names_for_sdf_groupings(
                        path_elements.clone(),
                        result,
                        value.as_object().unwrap(),
                    )
                }
            }
        }
    }
}

fn determine_global_names_for_affordance_or_data_qualities(
    path_elements: Vec<&str>,
    result: &mut HashSet<String>,
    parent_definition: &Map<String, Value>,
) {
    for affordance_type in ["sdfAction", "sdfProperty", "sdfEvent", "sdfData"] {
        determine_global_names_for_affordance_or_data_quality(
            path_elements.clone(),
            result,
            parent_definition,
            affordance_type,
        )
    }
}

fn determine_global_names_for_sdf_groupings(
    path_elements: Vec<&str>,
    result: &mut HashSet<String>,
    parent_definition: &Map<String, Value>,
) {
    for grouping_type in ["sdfThing", "sdfObject"] {
        determine_global_names_for_sdf_grouping(
            path_elements.clone(),
            result,
            parent_definition,
            grouping_type,
        )
    }
}

// TODO: Decide whether that is actually efficient enough
pub(crate) fn determine_global_names_for_model(sdf_model: &Map<String, Value>) -> HashSet<String> {
    let mut result = HashSet::new();

    // TODO: Handle errors
    let target_namespace_url = obtain_namespace_url(sdf_model).unwrap().unwrap() + "#";

    let path_elements = vec![target_namespace_url.as_str()];

    determine_global_names_for_sdf_groupings(path_elements.clone(), &mut result, sdf_model);
    determine_global_names_for_affordance_or_data_qualities(path_elements, &mut result, sdf_model);

    result
}

pub(crate) fn determine_global_names_for_models(
    sdf_models: Vec<&Map<String, Value>>,
) -> HashSet<String> {
    // TODO: Check whether this implementation can be optimized
    let mut result = HashSet::new();

    for sdf_model in sdf_models {
        let global_names = determine_global_names_for_model(sdf_model);

        for global_name in global_names.iter() {
            result.insert(global_name.clone());
        }
    }

    result
}

pub(crate) fn obtain_supplement_definitions(
    sdf_supplement: &Map<String, Value>,
) -> HashSet<String> {
    // TODO: Handle errors
    let namespace_url = obtain_namespace_url(sdf_supplement).unwrap().unwrap();

    let global_names = sdf_supplement
        .get("amend")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|x| {
            x.as_object()
                .unwrap()
                .iter()
                .map(|(key, _)| format!("{}{}", namespace_url, key))
        })
        .collect::<Vec<_>>();

    HashSet::from_iter(global_names)
}

pub(crate) fn find_model_matching_supplement<'a>(
    sdf_supplement: &'a Map<String, Value>,
    sdf_models: Vec<&'a Map<String, Value>>,
) -> actix_web::Result<Option<&'a Map<String, Value>>> {
    let lineage = get_info_block_field_value(sdf_supplement, "lineage");
    let target_version = get_info_block_field_value(sdf_supplement, "targetVersion");

    let supplement_namespace_url = obtain_namespace_url(sdf_supplement)?;

    let mut filtered_models = sdf_models
        .into_iter()
        .filter(|model| {
            let model_namespace_url = obtain_namespace_url(model).ok().flatten();

            let model_lineage = get_info_block_field_value(model, "lineage");
            let model_version = get_info_block_field_value(model, "version");

            lineage == model_lineage
                // TODO: Should we require version to be defined here?
                && target_version == model_version
                && supplement_namespace_url == model_namespace_url
        })
        .collect::<Vec<_>>();

    // TODO: Refactor!
    filtered_models.sort_by(|a, b| {
        let first_version = get_info_block_field_value(a, "version");
        let second_version = get_info_block_field_value(b, "version");

        if first_version.is_none() {
            if second_version.is_none() {
                return std::cmp::Ordering::Equal;
            } else {
                return std::cmp::Ordering::Less;
            }
        } else if second_version.is_none() {
            return std::cmp::Ordering::Greater;
        }

        let parsed_first_version = Version::parse(first_version.unwrap().as_str()).unwrap();
        let parsed_second_version = Version::parse(second_version.unwrap().as_str()).unwrap();

        parsed_first_version.cmp_precedence(&parsed_second_version)
    });

    let result = filtered_models.last().copied();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_rfc_9880_example() {
        let json = json!(
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

        let global_names = determine_global_names_for_model(json.as_object().unwrap());

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
        let json = json!(
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

        let global_names = determine_global_names_for_model(json.as_object().unwrap());

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
        let model1 = json!(
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

        let model2 = json!(
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

        let global_names = determine_global_names_for_models(vec![
            model1.as_object().unwrap(),
            model2.as_object().unwrap(),
        ]);

        let expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfObject/foo".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo/sdfProperty/bar".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo/sdfProperty/baz".to_string(),
        ]);

        assert_eq!(global_names, expected_result);
    }

    #[test]
    fn test_global_name_collisions() {
        let existing_model1 = json!(
          {
            "info": {
              "lineage": "foo"
            },
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

        let existing_model2 = json!(
          {
            "info": {
              "lineage": "bar"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "bar": {
                "sdfProperty": {
                  "foo": {},
                }
              }
            }
          }
        );

        let existing_global_names = determine_global_names_for_models(vec![
            existing_model1.as_object().unwrap(),
            existing_model2.as_object().unwrap(),
        ]);

        let first_expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfObject/foo".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo/sdfProperty/bar".to_string(),
            "https://example.com/capability/cap#/sdfObject/bar".to_string(),
            "https://example.com/capability/cap#/sdfObject/bar/sdfProperty/foo".to_string(),
        ]);

        assert_eq!(existing_global_names, first_expected_result);

        let new_model = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "bar": {
                "sdfProperty": {
                  "foo": {},
                }
              },
              "foo": {}
            }
          }
        );

        let second_expected_result = HashSet::<String>::from_iter(vec![
            "https://example.com/capability/cap#/sdfObject/bar".to_string(),
            "https://example.com/capability/cap#/sdfObject/bar/sdfProperty/foo".to_string(),
            "https://example.com/capability/cap#/sdfObject/foo".to_string(),
        ]);

        let colliding_global_names = determine_global_name_collisions(
            new_model.as_object().unwrap(),
            vec![
                existing_model1.as_object().unwrap(),
                existing_model2.as_object().unwrap(),
            ],
        );

        assert_eq!(colliding_global_names, second_expected_result);
    }

    #[test]
    fn test_supplement_definition_collection() {
        let sdf_supplement = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "amend": [
              {
                "#/sdfObject/Digital_Input": {
                  "delta": {
                    "id": 3200
                  }
                },
                "#/sdfObject/Digital_Input/sdfProperty/Digital_Input_State": {
                  "delta": {
                    "id": 5500
                  }
                }
              },
              {
                "#/sdfObject/Digital_Input/sdfProperty/Digital_Input_Counter": {
                  "delta": {
                    "id": 5501
                  }
                }
              }
            ]
          }
        );

        let supplement_definitions =
            obtain_supplement_definitions(sdf_supplement.as_object().unwrap());

        let expected_result = HashSet::<String>::from_iter(vec![
          "https://example.com/capability/cap#/sdfObject/Digital_Input/sdfProperty/Digital_Input_State".to_string(),
          "https://example.com/capability/cap#/sdfObject/Digital_Input/sdfProperty/Digital_Input_Counter".to_string(),
          "https://example.com/capability/cap#/sdfObject/Digital_Input".to_string(),
        ]);

        assert_eq!(supplement_definitions, expected_result);
    }

    #[test]
    fn test_supplement_model_association() {
        let model1 = json!(
          {
            "info": {
              "lineage": "foo"
            },
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

        let model2 = json!(
          {
            "info": {
              "lineage": "bar"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "bar": {
                "sdfProperty": {
                  "foo": {},
                }
              }
            }
          }
        );

        let sdf_models = vec![model1.as_object().unwrap(), model2.as_object().unwrap()];

        let sdf_supplement = json!(
          {
            "info": {
              "lineage": "foo"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "amend": [
              {
                "#/sdfObject/foo": {
                  "delta": {
                    "id": 3200
                  }
                },
                "#/sdfObject/foo/sdfProperty/bar": {
                  "delta": {
                    "id": 5500
                  }
                }
              },
            ]
          }
        );

        let found_model =
            find_model_matching_supplement(sdf_supplement.as_object().unwrap(), sdf_models)
                .unwrap()
                .unwrap();

        assert_eq!(found_model, model1.as_object().unwrap());
    }

    #[test]
    fn test_supplement_model_association_with_no_match() {
        let model = json!(
          {
            "info": {
              "lineage": "foo"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "bar": {
                "sdfProperty": {
                  "foo": {},
                }
              }
            }
          }
        );

        let sdf_models = vec![model.as_object().unwrap()];

        let sdf_supplement = json!(
          {
            "info": {
              "lineage": "bar"
            },
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "amend": []
          }
        );

        let found_model =
            find_model_matching_supplement(sdf_supplement.as_object().unwrap(), sdf_models)
                .unwrap();

        assert_eq!(found_model, None);
    }

    #[test]
    fn test_supplement_model_association_with_duplicates() {
        let model1 = json!(
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

        let model2 = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "sdfObject": {
              "bar": {
                "sdfProperty": {
                  "foo": {},
                }
              }
            }
          }
        );

        let sdf_models = vec![model1.as_object().unwrap(), model2.as_object().unwrap()];

        let sdf_supplement = json!(
          {
            "namespace": {
              "cap": "https://example.com/capability/cap"
            },
            "defaultNamespace": "cap",
            "amend": [
              {
                "#/sdfObject/foo": {
                  "delta": {
                    "id": 3200
                  }
                },
                "#/sdfObject/bar": {
                  "delta": {
                    "id": 3200
                  }
                },
              },
            ]
          }
        );

        let found_model =
            find_model_matching_supplement(sdf_supplement.as_object().unwrap(), sdf_models)
                .unwrap()
                .unwrap();

        // FIXME: Revisit the current behavior here
        assert_eq!(model2.as_object().unwrap(), found_model);
    }
}
