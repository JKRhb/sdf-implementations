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
use semver::Version;
use serde_json::json;

use crate::{AppState, handlers::verify_content_type, models::add_model_to_state};

fn verify_sdf_model_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_model_content_type")]
pub(crate) async fn post_model_handler<'a>(
    req: web::Json<serde_json::Map<String, serde_json::Value>>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let mut raw_json = serde_json::to_value(req.0).unwrap();
    let sdf_model = raw_json.as_object_mut().unwrap();

    let info_block = sdf_model.get_mut("info");

    if let Some(info_block) = info_block {
        let info_block = info_block.as_object();

        let version = info_block
            .and_then(|x| x.get("version"))
            .and_then(|x| x.as_str());

        if let Some(version) = version {
            Version::parse(version).map_err(|x| {
                actix_web::error::ErrorBadRequest(format!("Invalid version quality: {}", x))
            })?;
        }
    } else {
        sdf_model.insert("info".to_string(), json!({"version": "1.0.0"}));
    }

    let mut models = data.models.lock().unwrap();
    add_model_to_state(&mut models, sdf_model.clone())?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(serde_json::to_string(sdf_model).unwrap()))
}
