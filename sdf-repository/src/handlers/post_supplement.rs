// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse, Responder,
    error::{ErrorBadRequest, ErrorInternalServerError},
    guard::GuardContext,
    http::header::ContentType,
    post, web,
};
use sdf_data_structures::supplement::SdfSupplement;

use crate::{
    AppState,
    handlers::verify_content_type,
    models::{SdfModelEntry, find_model_matching_supplement},
};

pub fn verify_sdf_supplement_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf-supplement+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_supplement_content_type")]
async fn post_supplement_handler(
    supplement: web::Json<SdfSupplement>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let mut sdf_models = data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let sdf_supplement = supplement.0;

    let sdf_model = find_model_matching_supplement(
        &sdf_supplement,
        (*sdf_models).iter().map(|x| &x.model).collect::<Vec<_>>(),
    )?;

    let updated_model;

    if let Some(sdf_model) = sdf_model {
        updated_model = sdf_model
            .update_sdf_model(&sdf_supplement)
            .map_err(|_| ErrorBadRequest("Error updating SDF model."))?;
    } else {
        return Err(actix_web::error::ErrorBadRequest(
            "No matching SDF model found!".to_string(),
        ));
    }

    let namespace = updated_model
        .get_default_namespace_url()
        .ok_or(ErrorInternalServerError("Internal server error"))?;
    let current_version = updated_model
        .get_default_namespace_url()
        .ok_or(ErrorInternalServerError("Internal server error"))?;
    let lineage = updated_model.get_lineage();

    let payload = serde_json::to_string(&sdf_model)
        .map_err(|_| ErrorInternalServerError("Internal server error"))?;

    sdf_models.push(SdfModelEntry::new(
        updated_model,
        current_version,
        namespace,
        lineage,
    ));

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(payload))
}
