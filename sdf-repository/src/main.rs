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
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    handlers::{
        delete_models::delete_model_handler, get_model::get_model, get_models::get_models,
        post_model::post_model_handler, post_supplement::post_supplement_handler,
    },
    models::AppState,
    traits::QueryHandler,
    validators::basic_authentication_validator,
};

#[cfg(not(feature = "sqlx"))]
use std::sync::Mutex;

#[cfg(feature = "sqlx")]
use sqlx::PgPool;

mod config;
mod error;
mod handlers;
mod models;
mod traits;
mod validators;

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
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/docs/openapi.json", api)
            })
            .into_app()
    })
    .bind((config.bind_address, config.port))?
    .run()
    .await
}
