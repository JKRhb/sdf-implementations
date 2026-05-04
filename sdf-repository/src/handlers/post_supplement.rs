// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse, Responder, guard::GuardContext, http::header::ContentType, post, web,
};
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};

use crate::{
    API_TAG, AppState, create_example_model, create_example_supplement, error::SdfRepositoryError,
    handlers::verify_content_type, traits::QueryHandler,
};

pub fn verify_sdf_supplement_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf-supplement+json")
}

#[utoipa::path(
    tag = API_TAG,
    request_body(content = SdfSupplement, description = "SDF supplement that is supposed to update an existing SDF model", content_type = "application/sdf-supplement+json", example = create_example_supplement),
    responses(
        (status = 200, description = "Updated SDF model", body = SdfModel, example = create_example_model),
        (status = 401, description = "No matching SDF model has been found")
    )
)]
#[post("/supplements", guard = "verify_sdf_supplement_content_type")]
async fn post_supplement_handler(
    supplement: web::Json<SdfSupplement>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let updated_model = data.update_model(&supplement.0).await?;

    let payload = serde_json::to_string(&updated_model)?;

    let model_location =
        updated_model
            .get_default_namespace_url()
            .ok_or(SdfRepositoryError::InternalFailure(
                "Missing namespace".to_string(),
            ))?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .insert_header(("Location", model_location))
        .body(payload))
}
