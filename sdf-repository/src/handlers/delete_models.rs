// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{HttpRequest, HttpResponse, Responder, delete, http::header::ContentType, web};
use serde::Deserialize;

use crate::error::SdfRepositoryError;
use crate::{AppState, models::query_parameters::QueryParameters, traits::QueryHandler};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteModelQuery {
    lineage: Option<String>,
    min_version: Option<String>,
}

impl TryInto<QueryParameters> for (String, DeleteModelQuery) {
    type Error = SdfRepositoryError;

    fn try_into(self) -> Result<QueryParameters, SdfRepositoryError> {
        let delete_model_query = self.1;

        let namespace = self.0;
        let lineage = delete_model_query.lineage;

        let min_version = delete_model_query
            .min_version
            .map(TryInto::try_into)
            .transpose()?;

        Ok(QueryParameters::new(
            namespace,
            lineage,
            None,
            min_version,
            None,
            None,
            None,
        ))
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

    let deleted_models = data.delete_models(query_parameters.try_into()?).await?;

    if deleted_models.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    let response = serde_json::to_string(&deleted_models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
