// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse, Responder, error::ErrorInternalServerError, guard::GuardContext,
    http::header::ContentType, post, web,
};
use sdf_data_structures::model::SdfModel;
use semver::Version;

use crate::{AppState, handlers::verify_content_type, models::{AppStateQueryHandler, add_model_to_state}};

fn verify_sdf_model_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_model_content_type")]
pub(crate) async fn post_model_handler(
    req: web::Json<SdfModel>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let sdf_model = req.0;

    let inserted_model = data.insert_model(sdf_model).await.unwrap();

    // if let Some(info_block) = &sdf_model.info {
    //     if let Some(version) = &info_block.version {
    //         Version::parse(version).map_err(|x| {
    //             actix_web::error::ErrorBadRequest(format!("Invalid version quality: {}", x))
    //         })?;
    //     }
    // } else {
    //     sdf_model = sdf_model.update_version("1.0.0".to_string());
    // }

    let payload = serde_json::to_string(&inserted_model)
        .map_err(|_| ErrorInternalServerError("Internal server error"))?;

    // let mut models = data
    //     .models
    //     .lock()
    //     .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    // add_model_to_state(&mut models, sdf_model)?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(payload))
}
