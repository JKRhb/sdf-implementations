// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use ::serde::Deserialize;
use actix_web::{HttpRequest, HttpResponse, Responder, get, http::header::ContentType, web};
use sqlx::Error;

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters, SemanticVersion},
};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GetModelQuery {
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl TryInto<QueryParameters> for (String, GetModelQuery) {
    type Error = Error;

    fn try_into(self) -> Result<QueryParameters, Error> {
        let namespace = self.0;
        let get_model_query = self.1;

        let version: Option<SemanticVersion> =
            get_model_query.version.map(|x| x.try_into().unwrap());
        let min_version: Option<SemanticVersion> =
            get_model_query.min_version.map(|x| x.try_into().unwrap());
        let max_version: Option<SemanticVersion> =
            get_model_query.max_version.map(|x| x.try_into().unwrap());
        let exclusive_min_version: Option<SemanticVersion> = get_model_query
            .exclusive_min_version
            .map(|x| x.try_into().unwrap());
        let exclusive_max_version: Option<SemanticVersion> = get_model_query
            .exclusive_max_version
            .map(|x| x.try_into().unwrap());

        Ok(QueryParameters {
            namespace: namespace,
            lineage: get_model_query.lineage,
            version,
            min_version,
            max_version,
            exclusive_min_version,
            exclusive_max_version,
        })
    }
}

#[utoipa::path()]
#[get("/{tail:.*}")]
async fn get_model(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<GetModelQuery>,
) -> actix_web::Result<impl Responder> {
    let full_request_url = data.config.get_base_url() + req.path();

    let query_parameters = (full_request_url, query.0);

    let sdf_model = data
        .get_model(query_parameters.try_into().unwrap())
        .await
        .unwrap();

    let response = serde_json::to_string(&sdf_model)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
