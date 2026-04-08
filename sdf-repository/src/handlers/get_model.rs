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
    AppState, error::SdfRepositoryError, models::{AppStateQueryHandler, GetModelQuery, SdfModelEntry},
};

#[utoipa::path()]
#[get("/{tail:.*}")]
async fn get_model(
    data: web::Data<AppState>,
    query: web::Query<GetModelQuery>,
) -> actix_web::Result<impl Responder> {
    let sdf_model = data.get_model(query.0).unwrap();

    // let sdf_model =

    // let config = &req
    //     .app_data::<web::Data<AppState>>()
    //     .expect("Invalid app state!")
    //     .config;

    // let full_request_url = config.get_base_url() + req.path();

    // let models = &mut data
    //     .models
    //     .lock()
    //     .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

    // let mut matching_sdf_models = models
    //     .iter()
    //     .filter(|sdf_model_entry| {
    //         full_request_url == sdf_model_entry.namespace
    //             && model_parameters
    //                 .compare_with_model_entry(sdf_model_entry)
    //                 .unwrap_or(false)
    //     })
    //     .collect::<Vec<_>>();

    // matching_sdf_models.sort_by(|a, b| {
    //     let first_version = a.model.get_version();
    //     let second_version = b.model.get_version();

    //     if first_version.is_none() {
    //         if second_version.is_none() {
    //             return std::cmp::Ordering::Equal;
    //         } else {
    //             return std::cmp::Ordering::Less;
    //         }
    //     } else if second_version.is_none() {
    //         return std::cmp::Ordering::Greater;
    //     }

    //     let parsed_first_version = Version::parse(first_version.unwrap().as_str()).unwrap();
    //     let parsed_second_version = Version::parse(second_version.unwrap().as_str()).unwrap();

    //     parsed_first_version.cmp_precedence(&parsed_second_version)
    // });

    // if matching_sdf_models.is_empty() {
    //     return Err(ErrorNotFound(
    //         "Could not find an SDF model matching the namespace and specified criteria.",
    //     ));
    // }

    let response = serde_json::to_string(&sdf_model)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
