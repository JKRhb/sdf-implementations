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
use sdf_data_structures::model::SdfModelBuilder;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, sync::Mutex};
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    handlers::{
        delete_models::delete_model_handler, get_model::get_model, get_models::get_models,
        post_model::post_model_handler, post_supplement::post_supplement_handler,
    },
    models::SdfModelEntry,
    validators::basic_authentication_validator,
};

mod config;
mod error;
mod handlers;
mod models;
mod validators;

struct AppState {
    models: Mutex<Vec<SdfModelEntry>>,
    config: Config,
}

#[cfg(feature = "sqlx")]
async fn init_db(config: &Config) -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let rows_affected = sqlx::query("SELECT * FROM models")
        .execute(&pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        let foobar = &SdfModelBuilder::default().build().unwrap();

        sqlx::query(
            "INSERT INTO models (model, version, namespace, lineage) VALUES ($1, $2, $3, $4)",
        )
        .bind(sqlx::types::Json(foobar))
        .bind(&foobar.get_version())
        .bind(&foobar.get_default_namespace_url())
        .bind(&foobar.get_lineage())
        .execute(&pool)
        .await
        .unwrap();

        let foobar = &SdfModelBuilder::default().build().unwrap();

        sqlx::query(
            "INSERT INTO models (model, version, namespace, lineage) VALUES ($1, $2, $3, $4)",
        )
        .bind(sqlx::types::Json(foobar))
        .bind(&foobar.get_version())
        .bind(&foobar.get_default_namespace_url())
        .bind(&foobar.get_lineage())
        .execute(&pool)
        .await
        .unwrap();
    }

    Ok(())
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

    init_db(&config).await.ok();

    let pool = PgPool::connect(&config.database_url).await.unwrap();

    let sdf_model_entry = sqlx::query_as::<_, SdfModelEntry>("SELECT * FROM models")
        .fetch_one(&pool)
        .await
        .unwrap();

    println!("{:?}", sdf_model_entry);

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
