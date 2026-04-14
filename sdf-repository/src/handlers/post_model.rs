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

use crate::{
    AppState, handlers::verify_content_type, models::add_model_to_state, traits::QueryHandler,
};

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

    let payload = serde_json::to_string(&inserted_model)
        .map_err(|_| ErrorInternalServerError("Internal server error"))?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(payload))
}
