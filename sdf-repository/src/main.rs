// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{
    App, HttpServer,
    web::{self},
};
use actix_web_httpauth::middleware::HttpAuthentication;
use env_logger::Env;
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        SecurityRequirement,
        security::{Http, SecurityScheme},
    },
};
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handlers::{
        delete_models::delete_model_handler, get_model::get_model, get_models::get_models,
        post_model::post_model_handler, post_supplement::post_supplement_handler,
    },
    models::config::Config,
    persistence::{
        AppState,
        initial_models::{create_initial_model, create_initial_supplement},
    },
    traits::QueryHandler,
    validators::basic_authentication_validator,
};

#[cfg(not(feature = "sqlx"))]
use std::sync::Mutex;

#[cfg(feature = "sqlx")]
use sqlx::PgPool;

mod error;
mod handlers;
mod models;
mod persistence;
mod traits;
mod validators;

static API_TAG: &str = "API Resources";
static NAMESPACE_TAG: &str = "Namespace Resources";

fn create_example_model() -> SdfModel {
    let config = Config::init().expect("Invalid configuration.");

    create_initial_model(&config).unwrap()
}

fn create_example_supplement() -> SdfSupplement {
    let config = Config::init().expect("Invalid configuration.");

    create_initial_supplement(&config).unwrap()
}

fn create_example_models() -> Vec<SdfModel> {
    let config = Config::init().expect("Invalid configuration.");

    let example_model = create_initial_model(&config).unwrap();

    vec![example_model]
}

#[derive(OpenApi)]
#[openapi(
        info(
          title = "SDF Repository"
        ),
        tags(
            (name = NAMESPACE_TAG, description = "Resources associated with model namespaces."),
            (name = API_TAG, description = "Resources for the SDF repository API."),
        ),
        modifiers(&SecurityAddon)
    )]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let config = Config::init().expect("Invalid configuration.");

        if !config.basic_auth_enabled {
            return;
        }

        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "basic_security",
            SecurityScheme::Http(Http::new(utoipa::openapi::security::HttpAuthScheme::Basic)),
        )
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv::dotenv().ok();

    let config = Config::init().map_err(std::io::Error::other)?;

    #[cfg(feature = "sqlx")]
    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Unable to connect to database!");

    let app_state = web::Data::new(AppState {
        #[cfg(not(feature = "sqlx"))]
        models: Mutex::new(Vec::new()),

        config: config.clone(),

        #[cfg(feature = "sqlx")]
        database: pool,
    });

    app_state.clone().initialize().await?;

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
                scope::scope("/sdf").service(get_model).service(
                    scope::scope("")
                        .wrap(HttpAuthentication::basic(basic_authentication_validator))
                        .service(delete_model_handler),
                ),
            )
            .openapi_service(|api| {
                let mut updated_api = ApiDoc::openapi();

                updated_api.merge(api);

                if let Some(path_item) = updated_api.paths.paths.get_mut("/api/models")
                    && let Some(operation) = path_item.post.as_mut()
                    && config.basic_auth_enabled
                {
                    operation.security = Some(vec![SecurityRequirement::new(
                        "basic_security",
                        Vec::<String>::new(),
                    )]);
                }

                if let Some(path_item) = updated_api.paths.paths.get_mut("/api/supplements")
                    && let Some(operation) = path_item.post.as_mut()
                    && config.basic_auth_enabled
                {
                    operation.security = Some(vec![SecurityRequirement::new(
                        "basic_security",
                        Vec::<String>::new(),
                    )]);
                }

                if let Some(path_item) = updated_api.paths.paths.get_mut("/sdf/{tail}")
                    && let Some(operation) = path_item.delete.as_mut()
                    && config.basic_auth_enabled
                {
                    operation.security = Some(vec![SecurityRequirement::new(
                        "basic_security",
                        Vec::<String>::new(),
                    )]);
                }

                SwaggerUi::new("/swagger-ui/{_:.*}").url("/docs/openapi.json", updated_api)
            })
            .into_app()
            .service(web::redirect("/", "/swagger-ui/"))
    })
    .bind((config.bind_address, config.port))?
    .run()
    .await
}
