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

#[cfg(test)]
mod tests {
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
}
