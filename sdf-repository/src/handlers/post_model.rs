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
use sdf_data_structures::model::SdfModel;

use crate::{
    API_TAG, AppState, create_example_model, error::SdfRepositoryError,
    handlers::verify_content_type, traits::QueryHandler,
};

fn verify_sdf_model_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf+json")
}

#[utoipa::path(
    tag = API_TAG,
    request_body(content = SdfModel, description = "SDF model that is supposed to be inserted into the SDF Repository", content_type = "application/sdf+json", example = create_example_model),
    responses(
        (status = 201, description = "Inserted SDF model", body = [SdfModel], example = create_example_model),
        (status = 401, description = "The SDF model's lineage already exists.")
    )
)]
#[post("/models", guard = "verify_sdf_model_content_type")]
pub(crate) async fn post_model_handler(
    req: web::Json<SdfModel>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let sdf_model = req.0;

    let inserted_model = data.insert_model(sdf_model).await?;

    let payload = serde_json::to_string(&inserted_model)?;

    let model_location =
        inserted_model
            .get_default_namespace_url()
            .ok_or(SdfRepositoryError::InternalFailure(
                "Missing namespace".to_string(),
            ))?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .insert_header(("Location", model_location))
        .body(payload))
}
