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
