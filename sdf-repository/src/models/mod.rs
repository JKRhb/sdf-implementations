// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "sqlx"))]
pub(crate) mod in_memory;

#[cfg(feature = "sqlx")]
pub(crate) mod postgres;

#[cfg(not(feature = "sqlx"))]
use std::sync::Mutex;

#[cfg(feature = "sqlx")]
use sqlx::PgPool;

use crate::config::Config;

#[cfg(not(feature = "sqlx"))]
use crate::models::in_memory::SdfModelEntry;

pub(crate) struct AppState {
    #[cfg(not(feature = "sqlx"))]
    pub(crate) models: Mutex<Vec<SdfModelEntry>>,

    pub(crate) config: Config,

    #[cfg(feature = "sqlx")]
    pub(crate) database: PgPool,
}
