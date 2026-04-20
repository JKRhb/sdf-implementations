// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{HttpResponse, Responder, get, http::header::ContentType, web};
use serde::Deserialize;
use sqlx::Error;

use crate::{
    AppState,
    traits::{QueryHandler, QueryParameters, SemanticVersion},
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

impl TryInto<QueryParameters> for GetModelsQuery {
    type Error = Error;

    fn try_into(self) -> Result<QueryParameters, Error> {
        let version: Option<SemanticVersion> = self.version.map(|x| x.try_into().unwrap());
        let min_version: Option<SemanticVersion> = self.min_version.map(|x| x.try_into().unwrap());
        let max_version: Option<SemanticVersion> = self.max_version.map(|x| x.try_into().unwrap());
        let exclusive_min_version: Option<SemanticVersion> =
            self.exclusive_min_version.map(|x| x.try_into().unwrap());
        let exclusive_max_version: Option<SemanticVersion> =
            self.exclusive_max_version.map(|x| x.try_into().unwrap());

        Ok(QueryParameters {
            namespace: self.namespace,
            lineage: self.lineage,
            version,
            min_version,
            max_version,
            exclusive_min_version,
            exclusive_max_version,
        })
    }
}

#[utoipa::path()]
#[get("/models")]
pub(crate) async fn get_models(
    model_query: web::Query<GetModelsQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models = data
        .get_models(model_query.0.try_into().unwrap())
        .await
        .unwrap();

    let response = serde_json::to_string(&models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
