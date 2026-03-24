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
    AppState, error::SdfRepositoryError, handlers::compare_semantic_version, models::SdfModelEntry,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ModelQuery {
    namespace: String,
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl ModelQuery {
    fn compare_with_model_entry(
        &self,
        sdf_model_entry: &&SdfModelEntry,
    ) -> actix_web::Result<bool> {
        if sdf_model_entry.namespace != self.namespace || sdf_model_entry.lineage != self.lineage {
            return Ok(false);
        }

        let model_version = semver::Version::parse(&sdf_model_entry.version)
            .map_err(|_| SdfRepositoryError::InternalModelQueryError())?;

        for (query_version, ordering) in [
            (&self.version, vec![Ordering::Equal]),
            (&self.min_version, vec![Ordering::Greater, Ordering::Equal]),
            (&self.max_version, vec![Ordering::Less, Ordering::Equal]),
            (&self.exclusive_min_version, vec![Ordering::Greater]),
            (&self.exclusive_max_version, vec![Ordering::Less]),
        ] {
            if let Some(version) = query_version
                && !compare_semantic_version(&model_version, version, ordering)?
            {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[utoipa::path()]
#[get("/models")]
pub(crate) async fn get_models<'a>(
    model_query: web::Query<ModelQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models_entries = data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let models = models_entries
        .iter()
        .filter(|model_entry| {
            model_query
                .compare_with_model_entry(model_entry)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let response = serde_json::to_string(&models)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
