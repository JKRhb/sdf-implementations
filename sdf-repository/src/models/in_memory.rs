// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::sync::atomic::AtomicI32;

use actix_web::web;
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};

use crate::{
    error::SdfRepositoryError,
    models::AppState,
    traits::{QueryHandler, SemanticVersion},
};

static MODEL_ID_SEQ: AtomicI32 = AtomicI32::new(0);

#[derive(serde::Serialize, Debug, Clone)]
pub struct SdfModelEntry {
    id: i32,
    pub model: SdfModel,
    pub version: String,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl From<SdfModel> for SdfModelEntry {
    fn from(sdf_model: SdfModel) -> Self {
        let version = sdf_model.get_version().unwrap();
        let namespace = sdf_model.get_default_namespace_url().unwrap();
        let lineage = sdf_model.get_lineage();

        SdfModelEntry::new(sdf_model, version, namespace, lineage)
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
        let first_version: Option<SemanticVersion> = a
            .get_version()
            .and_then(|x| Some(SemanticVersion::try_from(x).unwrap()));
        let second_version: Option<SemanticVersion> = b
            .get_version()
            .and_then(|x| Some(SemanticVersion::try_from(x).unwrap()));

        match (first_version, second_version) {
            (None, None) => std::cmp::Ordering::Equal,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(first_version), Some(second_version)) => first_version.cmp(&second_version),
        }
    });

    let result = filtered_models.last().copied();

    Ok(result)
}

fn check_for_existing_lineage(
    new_sdf_model: &SdfModel,
    existing_sdf_models: Vec<&SdfModel>,
) -> Result<bool, SdfRepositoryError> {
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

impl QueryHandler for web::Data<AppState> {
    async fn initialize(self) -> Result<(), SdfRepositoryError> {
        todo!()
    }

    async fn delete_models(
        self,
        query: crate::traits::QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError> {
        let mut models_entries = self.models.lock().unwrap();

        let model_iterator = models_entries.iter().cloned();

        let (deleted_models, remaining_models): (Vec<SdfModelEntry>, Vec<SdfModelEntry>) =
            model_iterator.partition(|x| query.clone().filter_model(&x.model).unwrap());

        models_entries.clear();

        models_entries.append(&mut (remaining_models.into()));

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

        let filtered_models: Vec<SdfModel> = existing_sdf_models
            .into_iter()
            .filter(|x| query.clone().filter_model(&x.model).unwrap())
            .map(|x| x.model.clone())
            .collect();

        Ok(filtered_models)
    }

    async fn insert_model(&self, model: SdfModel) -> Result<SdfModel, SdfRepositoryError> {
        let mutex = self.models.lock().unwrap();

        let existing_sdf_models = mutex
            .iter()
            .map(|sdf_model_entry| &sdf_model_entry.model)
            .collect::<Vec<_>>();

        let lineage_exists = check_for_existing_lineage(&model, existing_sdf_models.clone())?;

        if lineage_exists {
            return Err(SdfRepositoryError::InternalModelQueryError());
        }

        let lineage = model.get_lineage();

        let models_from_different_lineage = existing_sdf_models
            .into_iter()
            .filter(|existing_sdf_model| lineage != existing_sdf_model.get_lineage())
            .collect::<Vec<_>>();

        let collisions = model.determine_global_name_collisions(models_from_different_lineage);

        let namespace =
            model
                .get_default_namespace_url()
                .ok_or(SdfRepositoryError::ModelQueryError(
                    "Missing namespace URL!".to_string(),
                ))?;
        let version = model
            .get_version()
            .ok_or(SdfRepositoryError::ModelQueryError(
                "Missing version!".to_string(),
            ))?;

        if collisions.is_empty() {
            self.models.lock().unwrap().push(SdfModelEntry::new(
                model.clone(),
                version,
                namespace,
                lineage,
            ));
            return Ok(model);
        }

        Err(SdfRepositoryError::ModelQueryError(
            "Definition collisions detected!".to_string(),
        ))
    }

    async fn update_model(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<SdfModel, SdfRepositoryError> {
        let mut mutex = self.models.lock().unwrap();

        let existing_sdf_models = mutex
            .iter()
            .map(|sdf_model_entry| &sdf_model_entry.model)
            .collect::<Vec<_>>();

        let model_matching_supplement =
            find_model_matching_supplement(sdf_supplement, existing_sdf_models)
                .unwrap()
                .unwrap()
                .clone();

        let new_model = model_matching_supplement
            .update_sdf_model(sdf_supplement)
            .unwrap();

        let sdf_model_entry = SdfModelEntry::from(new_model.clone());

        mutex.push(sdf_model_entry);

        Ok(new_model)
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
