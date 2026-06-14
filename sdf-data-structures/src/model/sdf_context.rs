// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{model::sdf_data::SdfData, traits::GlobalNameContributor};

use crate::util::default_bool_true;
use crate::util::skip_bool_true;

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
pub struct SdfContext {
    #[serde(flatten)]
    #[builder(default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub internal_data: SdfData,

    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub writable: bool,
}

impl GlobalNameContributor for SdfContext {
    const QUALITY_NAME: &'static str = "sdfContext";
}
