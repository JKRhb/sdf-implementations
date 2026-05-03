// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::supplement::SdfSupplement;

#[cfg(not(feature = "sqlx"))]
use crate::persistence::in_memory::SdfModelEntry;

#[cfg(feature = "sqlx")]
use sqlx::QueryBuilder;

use crate::{error::SdfRepositoryError, models::semantic_version::SemanticVersion};

#[derive(Debug, Clone)]
pub(crate) struct QueryParameters {
    namespace: String,
    lineage: Option<String>,
    version: Option<SemanticVersion>,
    min_version: Option<SemanticVersion>,
    max_version: Option<SemanticVersion>,
    exclusive_min_version: Option<SemanticVersion>,
    exclusive_max_version: Option<SemanticVersion>,
}

impl QueryParameters {
    pub fn new(
        namespace: String,
        lineage: Option<String>,
        version: Option<SemanticVersion>,
        min_version: Option<SemanticVersion>,
        max_version: Option<SemanticVersion>,
        exclusive_min_version: Option<SemanticVersion>,
        exclusive_max_version: Option<SemanticVersion>,
    ) -> Self {
        QueryParameters {
            namespace,
            lineage,
            version,
            min_version,
            max_version,
            exclusive_max_version,
            exclusive_min_version,
        }
    }

    #[cfg(feature = "sqlx")]
    pub fn create_query_builder<'a>(
        self,
        init: impl Into<String>,
    ) -> QueryBuilder<'a, sqlx::Postgres> {
        let mut query_builder = QueryBuilder::new(init);

        query_builder.push(" WHERE namespace = ");
        query_builder.push_bind(self.namespace);

        query_builder.push(" AND lineage IS NOT DISTINCT FROM ");
        query_builder.push_bind(self.lineage);

        for (comparator, semantic_version) in [
            ("=", self.version),
            (">=", self.min_version),
            ("<=", self.max_version),
            (">", self.exclusive_min_version),
            ("<", self.exclusive_max_version),
        ] {
            if let Some(semantic_version) = semantic_version {
                let version_vector: Vec<i32> = semantic_version.into();

                query_builder.push(format!(" AND version {} ", comparator));
                query_builder.push_bind(version_vector);
            }
        }

        query_builder.push(" ORDER BY version DESC");

        query_builder
    }

    #[cfg(not(feature = "sqlx"))]
    pub(crate) fn filter_model_entry(self, sdf_model_entry: &SdfModelEntry) -> bool {
        let model_version = sdf_model_entry.version;

        let namespace = sdf_model_entry.namespace.clone();
        let lineage = sdf_model_entry.lineage.clone();

        if self.namespace != namespace && self.lineage != lineage {
            return false;
        }

        if let Some(version) = &self.version
            && version != &model_version
        {
            return false;
        }

        if let Some(min_version) = &self.min_version
            && min_version > &model_version
        {
            return false;
        }

        if let Some(max_version) = &self.max_version
            && max_version < &model_version
        {
            return false;
        }

        if let Some(exclusive_min_version) = &self.exclusive_min_version
            && exclusive_min_version >= &model_version
        {
            return false;
        }

        if let Some(exclusive_max_version) = &self.exclusive_max_version
            && exclusive_max_version <= &model_version
        {
            return false;
        }

        true
    }
}

impl TryFrom<&SdfSupplement> for QueryParameters {
    type Error = SdfRepositoryError;

    fn try_from(sdf_supplement: &SdfSupplement) -> Result<Self, Self::Error> {
        let version = if let Some(version) = sdf_supplement.get_target_version() {
            let semantic_version: SemanticVersion = version.try_into()?;

            Some(semantic_version)
        } else {
            None
        };

        let namespace = sdf_supplement.get_default_namespace_url().ok_or(
            SdfRepositoryError::InputParameters(
                "No default namespace URL defined for SDF supplement.".to_string(),
            ),
        )?;
        let lineage = sdf_supplement.get_lineage();

        Ok(QueryParameters {
            namespace,
            lineage,
            version,
            min_version: None,
            max_version: None,
            exclusive_min_version: None,
            exclusive_max_version: None,
        })
    }
}
