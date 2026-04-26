// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use ::serde::Deserialize;
use actix_web::error::Error;
use actix_web::{HttpRequest, HttpResponse, Responder, get, http::header::ContentType, web};

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters},
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
        let get_model_query = self.1;

        let namespace = self.0;
        let lineage = get_model_query.lineage;

        let version = get_model_query
            .version
            .map(|version| version.try_into())
            .transpose()?;

        let min_version = get_model_query
            .min_version
            .map(|min_version| min_version.try_into())
            .transpose()?;

        let max_version = get_model_query
            .max_version
            .map(|max_version| max_version.try_into())
            .transpose()?;

        let exclusive_min_version = get_model_query
            .exclusive_min_version
            .map(|exclusive_min_version| exclusive_min_version.try_into())
            .transpose()?;

        let exclusive_max_version = get_model_query
            .exclusive_max_version
            .map(|exclusive_max_version| exclusive_max_version.try_into())
            .transpose()?;

        Ok(QueryParameters::new(
            namespace,
            lineage,
            version,
            min_version,
            max_version,
            exclusive_min_version,
            exclusive_max_version,
        ))
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

    let sdf_model = data.get_model(query_parameters.try_into()?).await?;

    let response = serde_json::to_string(&sdf_model)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
