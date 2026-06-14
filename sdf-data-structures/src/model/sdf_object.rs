// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};

use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    model::{
        affordances::{sdf_action::SdfAction, sdf_event::SdfEvent, sdf_property::SdfProperty},
        common_qualities::CommonQualities,
        sdf_context::SdfContext,
        sdf_data::SdfData,
    },
    traits::GlobalNameContributor,
};

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
#[serde(rename_all = "camelCase")]
#[ploidy(pointer(rename_all = "camelCase"))]
pub struct SdfObject {
    #[builder(setter(strip_option), default)]
    pub sdf_context: Option<HashMap<String, SdfContext>>,
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
    pub additional_qualities: Option<HashMap<String, Value>>,
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

        if let Some(sdf_context) = &self.sdf_context {
            for (key, value) in sdf_context.iter() {
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
