// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

//! Content-Formats for CoAP
//!
//! This module contains experimental CoAP Content-Format
//! assignments for SDF messages.
//!
//! The assigned numbers are temporary and will likely be
//! subject to change.

pub const SDF_SNAPSHOT_MESSAGE_CONTENT_FORMAT: u16 = 65000;
pub const SDF_DELTA_MESSAGE_CONTENT_FORMAT: u16 = 65001;
pub const SDF_CONSTRUCTION_MESSAGE_CONTENT_FORMAT: u16 = 65002;
pub const SDF_PATCH_MESSAGE_CONTENT_FORMAT: u16 = 65003;
