// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse, Responder, delete, error::ErrorInternalServerError, http::header::ContentType,
    web,
};
use serde::Deserialize;

use crate::{
    AppState,
    error::SdfRepositoryError,
    models::{AppStateQueryHandler, DeleteModelQuery, SdfModelEntry},
};

#[utoipa::path()]
#[delete("/{tail:.*}")]
pub(crate) async fn delete_model_handler(
    data: web::Data<AppState>,
    query: web::Query<DeleteModelQuery>,
) -> actix_web::Result<impl Responder> {
    let deleted_models = data.delete_models(query.0).unwrap();

    // let mut models_entries = data
    //     .models
    //     .lock()
    //     .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

    // let (deleted_entries, remaining_entries): (_, Vec<_>) = models_entries
    //     .iter()
    //     .cloned()
    //     .partition(|x| model_query.compare_with_model_entry(&x).unwrap_or(false));

    if deleted_models.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    let response =
        serde_json::to_string(&deleted_models)?;

    // models_entries.clear();

    // for remaining_entry in remaining_entries {
    //     models_entries.push(remaining_entry);
    // }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
