// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;

use actix_web::{
    HttpResponse, Responder, error::ErrorInternalServerError, get, http::header::ContentType, web,
};
use serde::Deserialize;

use crate::{
    AppState,
    error::SdfRepositoryError,
    models::{AppStateQueryHandler, GetModelsQuery, SdfModelEntry},
};

#[utoipa::path()]
#[get("/models")]
pub(crate) async fn get_models(
    model_query: web::Query<GetModelsQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models = data.get_models(model_query.0).unwrap();

    // let models_entries = data
    //     .models
    //     .lock()
    //     .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    // let models = models_entries
    //     .iter()
    //     .filter(|model_entry| {
    //         model_query
    //             .compare_with_model_entry(model_entry)
    //             .unwrap_or(false)
    //     })
    //     .collect::<Vec<_>>();
    let response = serde_json::to_string(&models)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
