use std::{collections::HashMap, error::Error};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{
    traits::SdfDataStructure,
    util::{default_bool_true, none_extra, skip_bool_true},
};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SdfSupplement {
    #[builder(setter(strip_option))]
    pub info: Option<InfoBlock>,
    #[builder(setter(into, strip_option), default)]
    pub namespace: Option<HashMap<String, String>>,
    #[builder(setter(into, strip_option), default)]
    pub default_namespace: Option<String>,
    #[builder(setter(into), default)]
    pub amend: Vec<HashMap<String, Amendment>>,
}

impl SdfDataStructure for SdfSupplement {
    fn namespace(&self) -> Option<&HashMap<String, String>> {
        self.namespace.as_ref()
    }

    fn default_namespace(&self) -> Option<&String> {
        self.default_namespace.as_ref()
    }
}

impl SdfSupplement {}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum PatchMethod {
    #[default]
    MergePatch,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Amendment {
    pub delta: Value,

    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub fix: bool,

    #[builder(setter(strip_option), default = "PatchMethod::MergePatch")]
    pub path_method: PatchMethod,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct InfoBlock {
    #[builder(setter(into, strip_option), default)]
    pub title: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub lineage: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub target_version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub modified: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub copyright: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub license: Option<String>,

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
