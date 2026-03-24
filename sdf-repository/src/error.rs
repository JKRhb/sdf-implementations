// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdfRepositoryError {
    #[error("error processing query parameters: {0}")]
    ModelQueryError(String),

    #[error("An internal error ocurred")]
    InternalModelQueryError(),
}

impl ResponseError for SdfRepositoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            SdfRepositoryError::ModelQueryError(_) => StatusCode::BAD_REQUEST,
            SdfRepositoryError::InternalModelQueryError() => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
