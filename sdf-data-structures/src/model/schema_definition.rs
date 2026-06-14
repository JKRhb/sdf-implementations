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
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::model::sdf_data::SdfData;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonPointee, JsonPointerTarget)]
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
#[derive(
    PartialEq, Serialize, Deserialize, Debug, Clone, Builder, JsonPointee, JsonPointerTarget,
)]
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
#[derive(
    PartialEq, Serialize, Deserialize, Debug, Clone, Builder, JsonPointee, JsonPointerTarget,
)]
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
#[derive(
    PartialEq, Serialize, Deserialize, Debug, Clone, Builder, JsonPointee, JsonPointerTarget,
)]
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
#[derive(
    PartialEq, Serialize, Deserialize, Debug, Clone, Builder, JsonPointee, JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema {
    #[builder(setter(into, strip_option), default)]
    pub required: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub properties: Option<HashMap<String, SdfData>>,
}
