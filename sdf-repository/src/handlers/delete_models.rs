// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::error::Error;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, http::header::ContentType, web};
use serde::Deserialize;

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters, SemanticVersion},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteModelQuery {
    lineage: Option<String>,
    min_version: Option<String>,
}

impl TryInto<QueryParameters> for (String, DeleteModelQuery) {
    type Error = Error;

    fn try_into(self) -> Result<QueryParameters, Error> {
        let namespace = self.0;
        let get_model_query = self.1;

        let min_version: Option<SemanticVersion> =
            get_model_query.min_version.map(|x| x.try_into().unwrap());

        Ok(QueryParameters {
            namespace: namespace,
            lineage: get_model_query.lineage,
            version: None,
            min_version,
            max_version: None,
            exclusive_min_version: None,
            exclusive_max_version: None,
        })
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

    let deleted_models = data
        .delete_models(query_parameters.try_into().unwrap())
        .await
        .unwrap();

    if deleted_models.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    let response = serde_json::to_string(&deleted_models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
