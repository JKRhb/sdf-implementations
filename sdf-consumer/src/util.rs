// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use serde_json::Value;

/// Custom parser function mapping an input string to a generic `serde_json::Value``.
pub(crate) fn parse_json_value(input_string: &str) -> anyhow::Result<Value> {
    let result = serde_json::from_str(input_string)?;

    Ok(result)
}
