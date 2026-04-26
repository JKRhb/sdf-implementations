// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::sync::atomic::AtomicI32;

use actix_web::web;
use itertools::Itertools;
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};

use crate::{
    error::SdfRepositoryError,
    models::{AppState, initial_models::create_initial_models},
    traits::{QueryHandler, SemanticVersion},
};

static MODEL_ID_SEQ: AtomicI32 = AtomicI32::new(0);

#[derive(serde::Serialize, Debug, Clone)]
pub struct SdfModelEntry {
    id: i32,
    pub model: SdfModel,
    pub version: SemanticVersion,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl TryFrom<SdfModel> for SdfModelEntry {
    type Error = SdfRepositoryError;

    fn try_from(sdf_model: SdfModel) -> Result<Self, Self::Error> {
        let version = sdf_model
            .get_version()
            .ok_or(SdfRepositoryError::ModelConversion(
                "Missing version quality".to_string(),
            ))?;

        let version: SemanticVersion = version.try_into()?;

        let namespace =
            sdf_model
                .get_default_namespace_url()
                .ok_or(SdfRepositoryError::ModelConversion(
                    "Invalid target namespace definition".to_string(),
                ))?;

        let lineage = sdf_model.get_lineage();

        Ok(SdfModelEntry::new(sdf_model, version, namespace, lineage))
    }
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
        version: SemanticVersion,
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

trait ModelFilter {
    fn find_model_matching_supplement(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<Option<SdfModelEntry>, SdfRepositoryError>;

    fn check_for_existing_lineage(
        &self,
        new_sdf_model: &SdfModel,
    ) -> Result<bool, SdfRepositoryError>;
}

impl ModelFilter for web::Data<AppState> {
    fn find_model_matching_supplement(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<Option<SdfModelEntry>, SdfRepositoryError> {
        let lineage = sdf_supplement.get_lineage();
        let target_version = sdf_supplement
            .get_target_version()
            .map(SemanticVersion::try_from)
            .transpose()?;
        let supplement_namespace_url = sdf_supplement.get_default_namespace_url();

        let mutex = self.models.lock().unwrap();

        Ok(mutex
            .iter()
            .filter(|model_entry| {
                lineage == model_entry.lineage
                    && target_version == Some(model_entry.version)
                    && supplement_namespace_url == Some(model_entry.namespace.clone())
            })
            .sorted_by(|a, b| a.version.cmp(&b.version))
            .last()
            .cloned())
    }

    fn check_for_existing_lineage(
        &self,
        new_sdf_model: &SdfModel,
    ) -> Result<bool, SdfRepositoryError> {
        let target_namespace_url = new_sdf_model.get_default_namespace_url();
        let lineage = new_sdf_model.get_lineage();

        let mutex = self.models.lock().unwrap();

        for existing_sdf_model_entry in mutex.iter() {
            let existing_target_namespace_url = Some(existing_sdf_model_entry.namespace.clone());
            let existing_lineage = existing_sdf_model_entry.lineage.clone();

            if target_namespace_url == existing_target_namespace_url && lineage == existing_lineage
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl QueryHandler for web::Data<AppState> {
    async fn initialize(self) -> Result<(), SdfRepositoryError> {
        let initial_models = create_initial_models(&self.config)?;

        for initial_model in initial_models {
            self.insert_model(initial_model).await?;
        }

        Ok(())
    }

    async fn delete_models(
        self,
        query: crate::traits::QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError> {
        let mut models_entries = self.models.lock().unwrap();

        let model_iterator = models_entries.iter().cloned();

        let (deleted_models, mut remaining_models): (Vec<SdfModelEntry>, Vec<SdfModelEntry>) =
            model_iterator.partition(|x| query.clone().filter_model_entry(x));

        models_entries.clear();

        models_entries.append(&mut remaining_models);

        Ok(deleted_models
            .iter()
            .map(|x| x.model.clone())
            .collect::<Vec<_>>())
    }

    async fn get_model(
        &self,
        query: crate::traits::QueryParameters,
    ) -> Result<SdfModel, SdfRepositoryError> {
        let first_result = self.get_models(query).await?.first().unwrap().clone();

        Ok(first_result)
    }

    async fn get_models(
        &self,
        query: crate::traits::QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError> {
        let mutex = self.models.lock().unwrap();

        let existing_sdf_models = mutex.iter().collect::<Vec<_>>();

        let filtered_models: Vec<_> = existing_sdf_models
            .iter()
            .filter(|x| query.clone().filter_model_entry(x))
            .map(|x| x.model.clone())
            .collect();

        Ok(filtered_models)
    }

    async fn insert_model(&self, model: SdfModel) -> Result<SdfModel, SdfRepositoryError> {
        let lineage_exists = self.check_for_existing_lineage(&model)?;

        if lineage_exists {
            return Err(SdfRepositoryError::ModelQuery(
                "Lineage already exists".to_string(),
            ));
        }

        let lineage = model.get_lineage();

        let mut mutex = self.models.lock().unwrap();

        let existing_sdf_models = mutex
            .iter()
            .map(|sdf_model_entry| &sdf_model_entry.model)
            .collect::<Vec<_>>();

        let models_from_different_lineage = existing_sdf_models
            .into_iter()
            .filter(|existing_sdf_model| lineage != existing_sdf_model.get_lineage())
            .collect::<Vec<_>>();

        let collisions = model.determine_global_name_collisions(models_from_different_lineage);

        let namespace = model
            .get_default_namespace_url()
            .ok_or(SdfRepositoryError::ModelQuery(
                "Missing namespace URL!".to_string(),
            ))?;

        let version = model.get_version().ok_or(SdfRepositoryError::ModelQuery(
            "Missing version!".to_string(),
        ))?;

        let version: SemanticVersion = version.try_into()?;

        if collisions.is_empty() {
            mutex.push(SdfModelEntry::new(
                model.clone(),
                version,
                namespace,
                lineage,
            ));
            return Ok(model);
        }

        Err(SdfRepositoryError::ModelQuery(
            "Definition collisions detected!".to_string(),
        ))
    }

    async fn update_model(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<SdfModel, SdfRepositoryError> {
        let model_matching_supplement =
            self.find_model_matching_supplement(sdf_supplement)?.ok_or(
                SdfRepositoryError::ModelQuery("Found no model matching supplement!".to_string()),
            )?;

        let new_model = model_matching_supplement
            .model
            .update_sdf_model(sdf_supplement)
            .unwrap();

        let sdf_model_entry = SdfModelEntry::try_from(new_model.clone())?;

        let mut mutex = self.models.lock().unwrap();

        mutex.push(sdf_model_entry);

        Ok(new_model)
    }
}

#[cfg(test)]
mod tests {
    // use std::collections::HashMap;

    // use sdf_data_structures::{
    //     model::{InfoBlockBuilder, SdfModelBuilder, SdfObjectBuilder, SdfProperty},
    //     supplement::{self, AmendmentBuilder, SdfSupplementBuilder},
    // };
    // use serde_json::json;

    // #[test]
    // fn test_supplement_model_association() {
    //     let model1 = SdfModelBuilder::default()
    //         .info(InfoBlockBuilder::default().lineage("foo").build().unwrap())
    //         .namespace(HashMap::from_iter(vec![(
    //             "cap".to_string(),
    //             "https://example.com/capability/cap".to_string(),
    //         )]))
    //         .default_namespace("cap")
    //         .sdf_object(HashMap::from([(
    //             "foo".to_string(),
    //             SdfObjectBuilder::default()
    //                 .sdf_property(HashMap::from([("bar".to_string(), SdfProperty::default())]))
    //                 .build()
    //                 .unwrap(),
    //         )]))
    //         .build()
    //         .unwrap();

    //     let model2 = SdfModelBuilder::default()
    //         .info(InfoBlockBuilder::default().lineage("bar").build().unwrap())
    //         .namespace(HashMap::from_iter(vec![(
    //             "cap".to_string(),
    //             "https://example.com/capability/cap".to_string(),
    //         )]))
    //         .default_namespace("cap")
    //         .sdf_object(HashMap::from([(
    //             "bar".to_string(),
    //             SdfObjectBuilder::default()
    //                 .sdf_property(HashMap::from([("foo".to_string(), SdfProperty::default())]))
    //                 .build()
    //                 .unwrap(),
    //         )]))
    //         .build()
    //         .unwrap();

    //     let sdf_models = vec![&model1, &model2];

    //     let sdf_supplement = SdfSupplementBuilder::default()
    //         .info(
    //             supplement::InfoBlockBuilder::default()
    //                 .lineage("bar")
    //                 .build()
    //                 .unwrap(),
    //         )
    //         .namespace(HashMap::from_iter(vec![(
    //             "cap".to_string(),
    //             "https://example.com/capability/cap".to_string(),
    //         )]))
    //         .default_namespace("cap")
    //         .amend(vec![
    //             HashMap::from([(
    //                 "#/sdfObject/foo".into(),
    //                 AmendmentBuilder::default()
    //                     .delta(json!(
    //                         {
    //                             "id": 3200
    //                         }
    //                     ))
    //                     .build()
    //                     .unwrap(),
    //             )]),
    //             HashMap::from([(
    //                 "#/sdfObject/foo/sdfProperty/bar".into(),
    //                 AmendmentBuilder::default()
    //                     .delta(json!(
    //                         {
    //                             "id": 5500
    //                         }
    //                     ))
    //                     .build()
    //                     .unwrap(),
    //             )]),
    //         ])
    //         .build()
    //         .unwrap();

    //     let found_model = find_model_matching_supplement(&sdf_supplement, sdf_models)
    //         .unwrap()
    //         .unwrap();

    //     assert_eq!(found_model, &model2);
    // }

    // #[test]
    // fn test_supplement_model_association_with_no_match() {
    //     let model = SdfModelBuilder::default()
    //         .info(InfoBlockBuilder::default().lineage("foo").build().unwrap())
    //         .namespace(HashMap::from_iter(vec![(
    //             "cap".to_string(),
    //             "https://example.com/capability/cap".to_string(),
    //         )]))
    //         .default_namespace("cap")
    //         .sdf_object(HashMap::from([(
    //             "bar".to_string(),
    //             SdfObjectBuilder::default()
    //                 .sdf_property(HashMap::from([("foo".to_string(), SdfProperty::default())]))
    //                 .build()
    //                 .unwrap(),
    //         )]))
    //         .build()
    //         .unwrap();

    //     let sdf_models = vec![&model];

    //     let sdf_supplement = SdfSupplementBuilder::default()
    //         .info(
    //             supplement::InfoBlockBuilder::default()
    //                 .lineage("bar")
    //                 .build()
    //                 .unwrap(),
    //         )
    //         .namespace(HashMap::from_iter(vec![(
    //             "cap".to_string(),
    //             "https://example.com/capability/cap".to_string(),
    //         )]))
    //         .default_namespace("cap")
    //         .build()
    //         .unwrap();

    //     let found_model = find_model_matching_supplement(&sdf_supplement, sdf_models).unwrap();

    //     assert_eq!(found_model, None);
    // }
}
