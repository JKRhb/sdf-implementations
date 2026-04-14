// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse, Responder, get, http::header::ContentType, web,
};
use serde::Deserialize;

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters},
};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetModelsQuery {
    namespace: String,
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl Into<QueryParameters> for GetModelsQuery {
    fn into(self) -> QueryParameters {
        todo!()
    }
}

#[utoipa::path()]
#[get("/models")]
pub(crate) async fn get_models(
    model_query: web::Query<GetModelsQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models = data.get_models(model_query.0.into()).await.unwrap();

    let response = serde_json::to_string(&models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
