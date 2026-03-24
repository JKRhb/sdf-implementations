use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, delete,
    dev::ServiceRequest,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get,
    guard::GuardContext,
    http::header::ContentType,
    post,
    web::{self},
};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use env_logger::Env;
use json_merge_patch::json_merge_patch;
use json_pointer::JsonPointer;
use log::{info, warn};
use semver::Version;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::{cmp::Ordering, sync::Mutex};
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    error::SdfRepositoryError,
    logic::{
        check_for_existing_lineage, determine_global_name_collisions,
        find_model_matching_supplement, get_info_block_field_value, obtain_namespace_url,
    },
    models::SdfModelEntry,
};

mod config;
mod error;
mod logic;
mod models;

struct AppState {
    models: Mutex<Vec<SdfModelEntry>>,
    config: Config,
}

fn add_model_to_state(
    models: &mut Vec<SdfModelEntry>,
    new_sdf_model: Map<String, Value>,
) -> actix_web::Result<()> {
    let existing_sdf_models = models
        .iter()
        .map(|sdf_model_entry| &sdf_model_entry.model)
        .collect::<Vec<_>>();

    let lineage_exists = check_for_existing_lineage(&new_sdf_model, existing_sdf_models.clone())?;

    if lineage_exists {
        return Err(actix_web::error::ErrorBadRequest("Lineage already exists!"));
    }

    let collisions = determine_global_name_collisions(&new_sdf_model, existing_sdf_models);

    let namespace = obtain_namespace_url(&new_sdf_model)?
        .ok_or(actix_web::error::ErrorBadRequest("Missing namespace URL!"))?;

    let lineage = get_info_block_field_value(&new_sdf_model, "lineage");
    let version = get_info_block_field_value(&new_sdf_model, "version")
        .ok_or(actix_web::error::ErrorBadRequest("Missing version!"))?;

    if collisions.is_empty() {
        models.push(SdfModelEntry::new(
            new_sdf_model.clone(),
            version,
            namespace,
            lineage,
        ));
        return Ok(());
    }

    Err(actix_web::error::ErrorBadRequest(
        "Definition collisions detected!",
    ))
}

pub fn verify_content_type(ctx: &GuardContext, expected_content_type: &str) -> bool {
    let content_type_value = ctx.head().headers().get("content-type");

    if let Some(content_type_value) = content_type_value {
        return content_type_value
            .to_str()
            .is_ok_and(|content_type| content_type == expected_content_type);
    }

    false
}

pub fn verify_sdf_model_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf+json")
}

pub fn verify_sdf_supplement_content_type(ctx: &GuardContext) -> bool {
    verify_content_type(ctx, "application/sdf-supplement+json")
}

#[utoipa::path()]
#[post("/models", guard = "verify_sdf_model_content_type")]
async fn post_model_handler<'a>(
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

fn compare_semantic_version(
    model_version: &Version,
    other_version: &str,
    precedence: Vec<Ordering>,
) -> actix_web::Result<bool> {
    let parsed_other_version = semver::Version::parse(other_version).map_err(|_| {
        SdfRepositoryError::ModelQueryError(format!(
            "Version {other_version} does not adhere to semantic versioning!"
        ))
    })?;

    Ok(precedence.contains(&model_version.cmp_precedence(&parsed_other_version)))
}

#[utoipa::path()]
#[get("/{tail:.*}")]
async fn model_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    model_parameters: web::Query<ModelParameters>,
) -> actix_web::Result<impl Responder> {
    let config = &req
        .app_data::<AppState>()
        .expect("Invalid app state!")
        .config;

    let full_request_url = config.get_base_url() + req.path();

    let models = &mut data.models.lock().unwrap();

    let mut matching_sdf_models = models
        .iter()
        .filter(|sdf_model_entry| {
            full_request_url == sdf_model_entry.namespace
                && model_parameters
                    .compare_with_model_entry(&sdf_model_entry)
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    matching_sdf_models.sort_by(|a, b| {
        let first_version = get_info_block_field_value(&a.model, "version");
        let second_version = get_info_block_field_value(&b.model, "version");

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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteModelQuery {
    lineage: Option<String>,
    min_version: Option<String>,
}

impl DeleteModelQuery {
    fn compare_with_model_entry(
        &self,
        sdf_model_entry: &&SdfModelEntry,
    ) -> actix_web::Result<bool> {
        if sdf_model_entry.lineage != self.lineage {
            return Ok(false);
        }

        if let Some(min_version) = &self.min_version {
            let model_version = semver::Version::parse(&sdf_model_entry.version)
                .map_err(|_| SdfRepositoryError::InternalModelQueryError())?;

            let parsed_min_version = semver::Version::parse(min_version).map_err(|_| {
                SdfRepositoryError::ModelQueryError(format!(
                    "Version {min_version} does not adhere to semantic versioning!"
                ))
            })?;

            return Ok(parsed_min_version <= model_version);
        }

        Ok(true)
    }
}

#[utoipa::path()]
#[delete("/{tail:.*}")]
async fn delete_model_handler(
    data: web::Data<AppState>,
    model_query: web::Query<DeleteModelQuery>,
) -> actix_web::Result<impl Responder> {
    let mut models_entries = data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

    let (deleted_entries, remaining_entries): (_, Vec<_>) = models_entries
        .iter()
        .cloned()
        .partition(|x| model_query.compare_with_model_entry(&x).unwrap_or(false));

    let response =
        serde_json::to_string(&deleted_entries.iter().map(|x| &x.model).collect::<Vec<_>>())?;

    if deleted_entries.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    models_entries.clear();

    for remaining_entry in remaining_entries {
        models_entries.push(remaining_entry);
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ModelQuery {
    namespace: String,
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl ModelQuery {
    fn compare_with_model_entry(
        &self,
        sdf_model_entry: &&SdfModelEntry,
    ) -> actix_web::Result<bool> {
        if sdf_model_entry.namespace != self.namespace || sdf_model_entry.lineage != self.lineage {
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
#[get("/models")]
async fn get_models<'a>(
    model_query: web::Query<ModelQuery>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let models_entries = data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let models = models_entries
        .iter()
        .filter(|model_entry| {
            model_query
                .compare_with_model_entry(model_entry)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let response = serde_json::to_string(&models)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}

async fn basic_authentication_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> actix_web::Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_data = req.app_data::<AppState>().expect("invalid app state");
    let config = &app_data.config;

    if !config.basic_auth_enabled {
        warn!("Skipping basic authentication as it is not enabled in the configuration!");
        return Ok(req);
    }

    if credentials.user_id() == config.username && credentials.password() == Some(&config.password)
    {
        info!("Basic authentication successful");
        Ok(req)
    } else {
        Err((
            ErrorUnauthorized("Registering, deleting, and updating models requires authorization"),
            req,
        ))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv::dotenv().ok();

    let config = Config::init().unwrap();

    let app_state = web::Data::new(AppState {
        models: Mutex::new(Vec::new()),
        config: config.clone(),
    });

    if config.basic_auth_enabled {
        if config.username.is_empty() {
            panic!("No username defined for basic authentication!")
        }

        if config.password.is_empty() {
            panic!("No password defined for basic authentication!")
        }
    }

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .app_data(app_state.clone())
            .service(
                scope::scope("/api").service(get_models).service(
                    scope::scope("")
                        .wrap(HttpAuthentication::basic(basic_authentication_validator))
                        .service(post_model_handler)
                        .service(post_supplement_handler),
                ),
            )
            .service(
                scope::scope("/sdf").service(model_handler).service(
                    scope::scope("")
                        .wrap(HttpAuthentication::basic(basic_authentication_validator))
                        .service(delete_model_handler),
                ),
            )
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/docs/openapi.json", api)
            })
            .into_app()
    })
    .bind((config.bind_address, config.port))?
    .run()
    .await
}
