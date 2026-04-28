// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::web;
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
use serde_json::Value;

use crate::{
    error::SdfRepositoryError,
    models::{AppState, initial_models::create_initial_models},
    traits::{QueryHandler, QueryParameters, SemanticVersion},
};

#[derive(serde::Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow, sqlx::Type))]
pub struct DatabaseRow {
    id: i32,
    pub model: Value,
    pub version: Vec<i32>,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl QueryHandler for web::Data<AppState> {
    async fn initialize(self) -> Result<(), SdfRepositoryError> {
        let pool = &self.database;

        sqlx::migrate!("./migrations").run(pool).await?;

        let rows_affected = sqlx::query("SELECT * FROM models")
            .execute(pool)
            .await?
            .rows_affected();

        let database_is_empty = rows_affected == 0;

        if database_is_empty {
            let initial_models = create_initial_models(&self.config)?;

            for initial_model in initial_models {
                self.insert_model(initial_model).await?;
            }
        }

        Ok(())
    }

    async fn get_model(&self, query: QueryParameters) -> Result<SdfModel, SdfRepositoryError> {
        let mut query_builder = query.create_query_builder("SELECT * FROM models");

        let query = query_builder.build_query_as::<DatabaseRow>();
        let query_result = query.fetch_optional(&self.database).await?;

        let model_json = query_result
            .map(|row| row.model)
            .ok_or(sqlx::Error::RowNotFound)?;

        serde_json::from_value::<SdfModel>(model_json).map_err(|error| {
            {
                sqlx::Error::Decode(
                    format!("Error while deserializing SDF model: {}", error).into(),
                )
            }
            .into()
        })
    }

    async fn update_model(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<SdfModel, SdfRepositoryError> {
        let target_model = self.get_model((sdf_supplement).try_into()?).await?;

        let updated_model = target_model
            .update_sdf_model(sdf_supplement)
            .map_err(|error| {
                SdfRepositoryError::ModelQuery(format!("Error while updating SDF model: {error}"))
            })?;

        self.insert_model(updated_model).await
    }

    async fn get_models(
        &self,
        query: QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError> {
        let mut query_builder = query.create_query_builder("SELECT * FROM models");

        let query = query_builder.build_query_as::<DatabaseRow>();
        let results: Result<Vec<SdfModel>, _> = query
            .fetch_all(&self.database)
            .await?
            .into_iter()
            .map(|x| serde_json::from_value(x.model))
            .collect();

        Ok(results?)
    }

    async fn delete_models(
        self,
        query: QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError> {
        let mut query_builder = query.create_query_builder("DELETE * FROM models");

        let query = query_builder.build_query_as::<DatabaseRow>();
        let results: Result<Vec<SdfModel>, _> = query
            .fetch_all(&self.database)
            .await?
            .into_iter()
            .map(|x| serde_json::from_value(x.model))
            .collect();

        Ok(results?)
    }

    async fn insert_model(&self, sdf_model: SdfModel) -> Result<SdfModel, SdfRepositoryError> {
        let version = if let Some(version) = sdf_model.get_version() {
            let semantic_version: SemanticVersion = version.try_into()?;

            semantic_version
        } else {
            return Err(SdfRepositoryError::ModelQuery(
                "Missing version definition in SDF Model.".to_string(),
            ));
        };

        let version_vector: Vec<i32> = version.into();

        sqlx::query(
            "INSERT INTO models (model, version, namespace, lineage) VALUES ($1, $2, $3, $4)",
        )
        .bind(sqlx::types::Json(&sdf_model))
        .bind(version_vector)
        .bind(sdf_model.get_default_namespace_url())
        .bind(sdf_model.get_lineage())
        .execute(&self.database)
        .await?;

        Ok(sdf_model)
    }
}
