// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::error::Error;
use actix_web::{HttpResponse, Responder, get, http::header::ContentType, web};
use sdf_data_structures::model::SdfModel;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{API_TAG, create_example_models};
use crate::{AppState, models::query_parameters::QueryParameters, traits::QueryHandler};

#[derive(Deserialize, Clone, IntoParams)]
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
        let version = self.version.map(TryInto::try_into).transpose()?;

        let min_version = self.min_version.map(TryInto::try_into).transpose()?;

        let max_version = self.max_version.map(TryInto::try_into).transpose()?;

        let exclusive_min_version = self
            .exclusive_min_version
            .map(TryInto::try_into)
            .transpose()?;

        let exclusive_max_version = self
            .exclusive_max_version
            .map(TryInto::try_into)
            .transpose()?;

        Ok(QueryParameters::new(
            self.namespace,
            self.lineage,
            version,
            min_version,
            max_version,
            exclusive_min_version,
            exclusive_max_version,
        ))
    }
}

#[utoipa::path(
    tag = API_TAG,
    responses(
        (status = 200, description = "Requested SDF models", body = [Vec<SdfModel>], example = create_example_models),
        (status = 404, description = "No matching SDF models have been found")
    ),
    params(
        GetModelsQuery,
    )
)]
#[get("/models")]
pub(crate) async fn get_models(
    model_query: web::Query<GetModelsQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models = data.get_models(model_query.0.try_into()?).await?;

    let response = serde_json::to_string(&models)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
