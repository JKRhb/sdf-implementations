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
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::util::none_extra;

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
    pub additional_qualities: Option<HashMap<String, Value>>,
}
