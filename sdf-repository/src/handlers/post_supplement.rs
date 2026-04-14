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
use sdf_data_structures::supplement::SdfSupplement;

use crate::{AppState, handlers::verify_content_type, traits::QueryHandler};

pub fn verify_sdf_supplement_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf-supplement+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_supplement_content_type")]
async fn post_supplement_handler(
    supplement: web::Json<SdfSupplement>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let updated_model = data.update_model(&supplement.0).await.unwrap();

    let payload = serde_json::to_string(&updated_model)
        .map_err(|_| ErrorInternalServerError("Internal server error"))?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(payload))
}
