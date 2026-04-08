// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{cmp::Ordering, sync::atomic::AtomicI32};

use actix_web::web;
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
use semver::Version;
use serde::Deserialize;
use sqlx::Error;
#[cfg(feature = "sqlx")]
use sqlx::PgPool;

use crate::{config::Config, error::SdfRepositoryError};

static MODEL_ID_SEQ: AtomicI32 = AtomicI32::new(0);

#[derive(serde::Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SdfModelEntry {
    id: i32,
    #[cfg_attr(feature = "sqlx", sqlx(json))]
    pub model: SdfModel,
    pub version: String,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl PartialOrd for SdfModelEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl PartialEq for SdfModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.namespace == other.namespace
            && self.lineage == other.lineage
    }
}

impl SdfModelEntry {
    pub fn new(
        model: SdfModel,
        version: String,
        namespace: String,
        lineage: Option<String>,
    ) -> SdfModelEntry {
        SdfModelEntry {
            id: Self::get_next_model_id(),
            model,
            lineage,
            namespace,
            version,
        }
    }

    fn get_next_model_id() -> i32 {
        MODEL_ID_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        MODEL_ID_SEQ.load(std::sync::atomic::Ordering::SeqCst)
    }
}

pub(crate) fn add_model_to_state(
    models: &mut Vec<SdfModelEntry>,
    new_sdf_model: SdfModel,
) -> actix_web::Result<()> {
    let existing_sdf_models = models
        .iter()
        .map(|sdf_model_entry| &sdf_model_entry.model)
        .collect::<Vec<_>>();

    let lineage_exists = check_for_existing_lineage(&new_sdf_model, existing_sdf_models.clone())?;

    if lineage_exists {
        return Err(actix_web::error::ErrorBadRequest("Lineage already exists!"));
    }

    let lineage = new_sdf_model.get_lineage();

    let models_from_different_lineage = existing_sdf_models
        .into_iter()
        .filter(|existing_sdf_model| lineage != existing_sdf_model.get_lineage())
        .collect::<Vec<_>>();

    let collisions = new_sdf_model.determine_global_name_collisions(models_from_different_lineage);

    let namespace = new_sdf_model
        .get_default_namespace_url()
        .ok_or(actix_web::error::ErrorBadRequest("Missing namespace URL!"))?;
    let version = new_sdf_model
        .get_version()
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

pub(crate) fn check_for_existing_lineage(
    new_sdf_model: &SdfModel,
    existing_sdf_models: Vec<&SdfModel>,
) -> actix_web::Result<bool> {
    let target_namespace_url = new_sdf_model.get_default_namespace_url();
    let lineage = new_sdf_model.get_lineage();

    for existing_sdf_model in existing_sdf_models {
        let existing_target_namespace_url = existing_sdf_model.get_default_namespace_url();
        let existing_lineage = existing_sdf_model.get_lineage();

        if target_namespace_url == existing_target_namespace_url && lineage == existing_lineage {
            return Ok(true);
        }
    }

    Ok(false)
}

pub(crate) fn find_model_matching_supplement<'a>(
    sdf_supplement: &'a SdfSupplement,
    sdf_models: Vec<&'a SdfModel>,
) -> actix_web::Result<Option<&'a SdfModel>> {
    let lineage = sdf_supplement.get_lineage();
    let target_version = sdf_supplement.get_target_version();
    let supplement_namespace_url = sdf_supplement.get_default_namespace_url();

    let mut filtered_models = sdf_models
        .into_iter()
        .filter(|model| {
            let model_namespace_url = model.get_default_namespace_url();

            let model_lineage = model.get_lineage();
            let model_version = model.get_version();

            lineage == model_lineage
                && target_version == model_version
                && supplement_namespace_url == model_namespace_url
        })
        .collect::<Vec<_>>();

    filtered_models.sort_by(|a, b| {
        let first_version = a
            .get_version()
            .and_then(|x| Version::parse(x.as_str()).ok());
        let second_version = b
            .get_version()
            .and_then(|x| Version::parse(x.as_str()).ok());

        match (first_version, second_version) {
            (None, None) => std::cmp::Ordering::Equal,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(first_version), Some(second_version)) => {
                first_version.cmp_precedence(&second_version)
            }
        }
    });

    let result = filtered_models.last().copied();

    Ok(result)
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetModelsQuery {
    namespace: String,
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl GetModelsQuery {
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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetModelQuery {
    lineage: Option<String>,
    version: Option<String>,
    min_version: Option<String>,
    max_version: Option<String>,
    exclusive_min_version: Option<String>,
    exclusive_max_version: Option<String>,
}

impl GetModelQuery {
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteModelQuery {
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

pub(crate) struct AppState {
    #[cfg(not(feature = "sqlx"))]
    pub(crate) models: Mutex<Vec<SdfModelEntry>>,

    pub(crate) config: Config,

    #[cfg(feature = "sqlx")]
    pub(crate) database: PgPool,
}

pub(crate) trait AppStateQueryHandler {
    async fn init_database(self) -> Result<(), Error>;

    fn delete_models(self, query: DeleteModelQuery) -> Result<Vec<SdfModel>, Error>;

    fn get_model(self, query: GetModelQuery) -> Result<SdfModel, Error>;

    fn get_models(self, query: GetModelsQuery) -> Result<Vec<SdfModel>, Error>;

    async fn insert_model(self, model: SdfModel) -> Result<SdfModel, Error>;

    async fn update_model(self, model: SdfSupplement) -> Result<SdfModel, Error>;
}

impl AppStateQueryHandler for web::Data<AppState> {
    async fn init_database(self) -> Result<(), Error> {
        let pool = &self.database;

        sqlx::migrate!("./migrations").run(pool).await?;

        let rows_affected = sqlx::query("SELECT * FROM models")
            .execute(pool)
            .await?
            .rows_affected();

        let database_is_empty = rows_affected == 0;

        if database_is_empty {
            use sdf_data_structures::model::SdfModel;
            use serde_json::json;

            let mut namespace_url = self.config.get_base_url();

            namespace_url.push_str("/sdf/sensor");

            let initial_model = serde_json::from_value::<SdfModel>(json!({
            "info": {
                "lineage": "foobar",
                "version": "1.1.0"
            },
            "namespace": {
                "sensors": namespace_url
            },
            "defaultNamespace": "sensors",
            "sdfObject": {
                "envSensor": {
                    "sdfContext": {
                        "ipAdress": {
                            "type": "string"
                        },
                        "deviceName": {
                            "type": "string"
                        },
                        "unit": {
                            "type": "string"
                        }
                    },
                    "sdfProperty": {
                        "temperature": {
                            "type": "string",
                            "sdfProtocolMap": {
                                "coap": {
                                    "sdfParameters": {
                                        "ipAddress": "#/sdfObject/envSensor/sdfContext/ipAddress"
                                    },
                                    "sdfOperations": {
                                        "read": {
                                            "method": "GET",
                                            "href": "/temperature",
                                            "contentType": [60],
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
        .unwrap();

            self.insert_model(initial_model).await?;

            // insert_model(pool, &initial_model);
        }

        Ok(())
    }

    async fn insert_model(self, model: SdfModel) -> Result<SdfModel, Error> {
        sqlx::query(
            "INSERT INTO models (model, version, namespace, lineage) VALUES ($1, $2, $3, $4)",
        )
        .bind(sqlx::types::Json(&model))
        .bind(&model.get_version())
        .bind(&model.get_default_namespace_url())
        .bind(&model.get_lineage())
        .execute(&self.database)
        .await?;

        Ok(model)
    }

    async fn update_model(self, model: SdfSupplement) -> Result<SdfModel, Error> {
        todo!()
    }

    fn get_models(self, query: GetModelsQuery) -> Result<Vec<SdfModel>, sqlx::Error> {
        todo!()
    }

    fn get_model(self, query: GetModelQuery) -> Result<SdfModel, Error> {
        todo!()
    }

    fn delete_models(self, query: DeleteModelQuery) -> Result<Vec<SdfModel>, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use sdf_data_structures::{
        model::{InfoBlockBuilder, SdfModelBuilder, SdfObjectBuilder, SdfProperty},
        supplement::{self, AmendmentBuilder, SdfSupplementBuilder},
    };
    use serde_json::json;

    use super::*;

    #[test]
    fn test_supplement_model_association() {
        let model1 = SdfModelBuilder::default()
            .info(InfoBlockBuilder::default().lineage("foo").build().unwrap())
            .namespace(HashMap::from_iter(vec![(
                "cap".to_string(),
                "https://example.com/capability/cap".to_string(),
            )]))
            .default_namespace("cap")
            .sdf_object(HashMap::from([(
                "foo".to_string(),
                SdfObjectBuilder::default()
                    .sdf_property(HashMap::from([("bar".to_string(), SdfProperty::default())]))
                    .build()
                    .unwrap(),
            )]))
            .build()
            .unwrap();

        let model2 = SdfModelBuilder::default()
            .info(InfoBlockBuilder::default().lineage("bar").build().unwrap())
            .namespace(HashMap::from_iter(vec![(
                "cap".to_string(),
                "https://example.com/capability/cap".to_string(),
            )]))
            .default_namespace("cap")
            .sdf_object(HashMap::from([(
                "bar".to_string(),
                SdfObjectBuilder::default()
                    .sdf_property(HashMap::from([("foo".to_string(), SdfProperty::default())]))
                    .build()
                    .unwrap(),
            )]))
            .build()
            .unwrap();

        let sdf_models = vec![&model1, &model2];

        let sdf_supplement = SdfSupplementBuilder::default()
            .info(
                supplement::InfoBlockBuilder::default()
                    .lineage("bar")
                    .build()
                    .unwrap(),
            )
            .namespace(HashMap::from_iter(vec![(
                "cap".to_string(),
                "https://example.com/capability/cap".to_string(),
            )]))
            .default_namespace("cap")
            .amend(vec![
                HashMap::from([(
                    "#/sdfObject/foo".into(),
                    AmendmentBuilder::default()
                        .delta(json!(
                            {
                                "id": 3200
                            }
                        ))
                        .build()
                        .unwrap(),
                )]),
                HashMap::from([(
                    "#/sdfObject/foo/sdfProperty/bar".into(),
                    AmendmentBuilder::default()
                        .delta(json!(
                            {
                                "id": 5500
                            }
                        ))
                        .build()
                        .unwrap(),
                )]),
            ])
            .build()
            .unwrap();

        let found_model = find_model_matching_supplement(&sdf_supplement, sdf_models)
            .unwrap()
            .unwrap();

        assert_eq!(found_model, &model2);
    }

    #[test]
    fn test_supplement_model_association_with_no_match() {
        let model = SdfModelBuilder::default()
            .info(InfoBlockBuilder::default().lineage("foo").build().unwrap())
            .namespace(HashMap::from_iter(vec![(
                "cap".to_string(),
                "https://example.com/capability/cap".to_string(),
            )]))
            .default_namespace("cap")
            .sdf_object(HashMap::from([(
                "bar".to_string(),
                SdfObjectBuilder::default()
                    .sdf_property(HashMap::from([("foo".to_string(), SdfProperty::default())]))
                    .build()
                    .unwrap(),
            )]))
            .build()
            .unwrap();

        let sdf_models = vec![&model];

        let sdf_supplement = SdfSupplementBuilder::default()
            .info(
                supplement::InfoBlockBuilder::default()
                    .lineage("bar")
                    .build()
                    .unwrap(),
            )
            .namespace(HashMap::from_iter(vec![(
                "cap".to_string(),
                "https://example.com/capability/cap".to_string(),
            )]))
            .default_namespace("cap")
            .build()
            .unwrap();

        let found_model = find_model_matching_supplement(&sdf_supplement, sdf_models).unwrap();

        assert_eq!(found_model, None);
    }
}
