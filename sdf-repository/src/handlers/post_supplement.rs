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
use json_merge_patch::json_merge_patch;
use json_pointer::JsonPointer;
use semver::Version;

use crate::{
    AppState,
    handlers::verify_content_type,
    models::{
        SdfModelEntry, find_model_matching_supplement, get_info_block_field_value,
        obtain_namespace_url,
    },
};

#[derive(PartialEq, PartialOrd, Debug)]
enum NewVersionType {
    Major = 3,
    Minor = 2,
    Patch = 1,
    Unchanged = 0,
}

fn check_for_backwards_compatibility(json_pointer: &String) -> bool {
    // TODO: Double-check whether this approach works
    let minor_change_keywords = vec![
        "#", // Top-level definitions
        "sdfThing",
        "sdfObject",
        "sdfProperty",
        "sdfAction",
        "sdfEvent",
        "sdfData",
        "label",
        "description",
        "$comment",
    ];

    minor_change_keywords.contains(&json_pointer.split("/").last().unwrap())
}

pub fn verify_sdf_supplement_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf-supplement+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_supplement_content_type")]
async fn post_supplement_handler(
    supplement: web::Json<serde_json::Map<String, serde_json::Value>>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let mut sdf_models = data.models.lock().unwrap();
    let sdf_supplement = supplement.0;

    let sdf_model = find_model_matching_supplement(
        &sdf_supplement,
        (*sdf_models).iter().map(|x| &x.model).collect::<Vec<_>>(),
    )?;

    if sdf_model.is_none() {
        return Err(actix_web::error::ErrorBadRequest(
            "No matching SDF model found!".to_string(),
        ));
    }

    let mut new_model = serde_json::Value::Object(sdf_model.unwrap().clone());

    let current_version = new_model
        .get("info")
        .unwrap()
        .get("version")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let amendments = sdf_supplement
        .get("amend")
        .ok_or(actix_web::error::ErrorBadRequest("Missing amend member"))
        .map(|x| x.as_array())?
        .ok_or(actix_web::error::ErrorBadRequest(
            "Validation error: amend is not an array",
        ))?;

    let mut overall_new_version_type = NewVersionType::Unchanged;

    for amendment in amendments {
        let amendment_map = amendment
            .as_object()
            .ok_or(actix_web::error::ErrorBadRequest(
                "Amendment member is not an object.",
            ))?;

        for (key, value) in amendment_map.iter() {
            let delta = value
                .get("delta")
                .ok_or(actix_web::error::ErrorBadRequest("Missing delta quality"))?;

            let type_of_this_change: NewVersionType;

            let fix = value.get("fix").and_then(|x| x.as_bool()).unwrap_or(false);

            if fix {
                type_of_this_change = NewVersionType::Patch;
            } else {
                let backwards_compatible_change = check_for_backwards_compatibility(key);

                if backwards_compatible_change {
                    type_of_this_change = NewVersionType::Minor;
                } else {
                    type_of_this_change = NewVersionType::Major;
                }
            }

            if type_of_this_change > overall_new_version_type {
                overall_new_version_type = type_of_this_change;
            }

            let ptr = key.parse::<JsonPointer<_, _>>().unwrap();

            let target_definition = ptr.get_mut(&mut new_model).unwrap();

            json_merge_patch(target_definition, delta);
        }
    }

    let mut current_version = Version::parse(&current_version).unwrap();

    match overall_new_version_type {
        NewVersionType::Major => current_version.major += 1,
        NewVersionType::Minor => current_version.minor += 1,
        NewVersionType::Patch => current_version.patch += 1,
        _ => {}
    }

    new_model
        .get_mut("info")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("version".to_string(), current_version.to_string().into());

    let namespace = obtain_namespace_url(new_model.as_object().unwrap())?.unwrap();
    let lineage = get_info_block_field_value(new_model.as_object().unwrap(), "lineage");

    sdf_models.push(SdfModelEntry::new(
        new_model.as_object().unwrap().clone(),
        current_version.to_string(),
        namespace,
        lineage,
    ));

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&new_model).unwrap()))
}
