// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Map, Value};

pub fn none_extra<'de, D>(deserializer: D) -> Result<Option<HashMap<String, Value>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Map::deserialize(deserializer)?;

    Ok((!s.is_empty()).then_some(HashMap::from_iter(s)))
}

/// Helper function to return a true boolean value as a default during serialization.
#[inline]
pub(crate) fn default_bool_true() -> bool {
    true
}

/// Helper function to return a false boolean value as a default during serialization.
#[inline]
pub(crate) fn default_bool_false() -> bool {
    false
}

/// Helper function for skipping the serialization of a true boolean value.
#[inline]
pub(crate) fn skip_bool_true(value: &bool) -> bool {
    *value
}

/// Helper function for skipping the serialization of a false boolean value.
#[inline]
pub(crate) fn skip_bool_false(value: &bool) -> bool {
    !*value
}
