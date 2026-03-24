// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;

use ::serde::Deserialize;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::header::ContentType,
    web,
};
use semver::Version;

use crate::{
    AppState, error::SdfRepositoryError, handlers::compare_semantic_version, models::SdfModelEntry,
};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ModelParameters {
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl ModelParameters {
    fn compare_with_model_entry(
        &self,
        sdf_model_entry: &&SdfModelEntry,
    ) -> actix_web::Result<bool> {
        if sdf_model_entry.lineage != self.lineage {
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
#[get("/{tail:.*}")]
async fn get_model(
    req: HttpRequest,
    data: web::Data<AppState>,
    model_parameters: web::Query<ModelParameters>,
) -> actix_web::Result<impl Responder> {
    let config = &req
        .app_data::<AppState>()
        .expect("Invalid app state!")
        .config;

    let full_request_url = config.get_base_url() + req.path();

    let models = &mut data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

    let mut matching_sdf_models = models
        .iter()
        .filter(|sdf_model_entry| {
            full_request_url == sdf_model_entry.namespace
                && model_parameters
                    .compare_with_model_entry(sdf_model_entry)
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    matching_sdf_models.sort_by(|a, b| {
        let first_version = a.model.get_version();
        let second_version = b.model.get_version();

        if first_version.is_none() {
            if second_version.is_none() {
                return std::cmp::Ordering::Equal;
            } else {
                return std::cmp::Ordering::Less;
            }
        } else if second_version.is_none() {
            return std::cmp::Ordering::Greater;
        }

        let parsed_first_version = Version::parse(first_version.unwrap().as_str()).unwrap();
        let parsed_second_version = Version::parse(second_version.unwrap().as_str()).unwrap();

        parsed_first_version.cmp_precedence(&parsed_second_version)
    });

    if matching_sdf_models.is_empty() {
        return Err(ErrorNotFound(
            "Could not find an SDF model matching the namespace and specified criteria.",
        ));
    }

    let response = serde_json::to_string(&matching_sdf_models.last().unwrap().model)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
