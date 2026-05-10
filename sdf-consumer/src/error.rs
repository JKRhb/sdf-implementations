// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct SdfConsumerError {
    pub error_message: String,
}

impl std::error::Error for SdfConsumerError {}

impl Display for SdfConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to resolved URI for prefix: {}.",
            self.error_message,
        )
    }
}
