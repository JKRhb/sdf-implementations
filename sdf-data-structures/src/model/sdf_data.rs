// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    model::{common_qualities::CommonQualities, schema_definition::SchemaDefinition},
    traits::GlobalNameContributor,
};

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
    pub additional_qualities: Option<HashMap<String, Value>>,
}

fn deserialize_additional_sdf_data<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, Value>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let mut deserialized_map = Map::deserialize(deserializer)?;
    deserialized_map.retain(|key, _| key != "type");
    Ok((!deserialized_map.is_empty()).then_some(HashMap::from_iter(deserialized_map)))
}

impl GlobalNameContributor for SdfData {
    const QUALITY_NAME: &'static str = "sdfData";
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

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
}
