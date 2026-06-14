// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdfDataStructureError {
    #[error("resolving the target namespace failed: {0}")]
    TargetNamespaceError(String),
}
