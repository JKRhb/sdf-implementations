// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{HttpRequest, HttpResponse, Responder, delete, http::header::ContentType, web};
use serde::Deserialize;

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteModelQuery {
    lineage: Option<String>,
    min_version: Option<String>,
}

impl Into<QueryParameters> for (String, DeleteModelQuery) {
    fn into(self) -> QueryParameters {
        todo!()
    }
}

#[utoipa::path()]
#[delete("/{tail:.*}")]
pub(crate) async fn delete_model_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<DeleteModelQuery>,
) -> actix_web::Result<impl Responder> {
    let full_request_url = data.config.get_base_url() + req.path();

    let query_parameters = (full_request_url, query.0);

    let deleted_models = data.delete_models(query_parameters.into()).await.unwrap();

    if deleted_models.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    let response = serde_json::to_string(&deleted_models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
