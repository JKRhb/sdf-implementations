pub mod protocol_mappings;

use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{
    supplement::SdfSupplement,
    traits::{GlobalNameContributor, SdfDataStructure},
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

impl SdfModel {
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

impl GlobalNameContributor for SdfAction {
    const QUALITY_NAME: &'static str = "sdfAction";
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
}
