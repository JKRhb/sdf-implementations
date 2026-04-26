// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{ResponseError, http::StatusCode};
#[cfg(feature = "sqlx")]
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdfRepositoryError {
    #[error("Error processing query parameters: {0}")]
    ModelQuery(String),

    #[error("An error ocurred while interacting with the database: {0}")]
    Database(String),

    #[error("An error occurred during the serialization or deserialization of JSON: {0}")]
    Json(String),

    #[error("An error occurred during the internal conversion of an SDF model: {0}")]
    ModelConversion(String),
}

impl ResponseError for SdfRepositoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            SdfRepositoryError::ModelQuery(_) => StatusCode::BAD_REQUEST,
            SdfRepositoryError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SdfRepositoryError::Json(_) => StatusCode::BAD_REQUEST,
            SdfRepositoryError::ModelConversion(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<serde_json::Error> for SdfRepositoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

impl From<SdfRepositoryError> for std::io::Error {
    fn from(error: SdfRepositoryError) -> Self {
        std::io::Error::other(error)
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::error::Error> for SdfRepositoryError {
    fn from(error: sqlx::error::Error) -> Self {
        SdfRepositoryError::Database(error.to_string())
    }
}

#[cfg(feature = "sqlx")]
impl From<MigrateError> for SdfRepositoryError {
    fn from(error: MigrateError) -> Self {
        SdfRepositoryError::Database(error.to_string())
    }
}
