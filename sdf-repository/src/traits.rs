// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
#[cfg(feature = "sqlx")]
use sqlx::QueryBuilder;

#[cfg(not(feature = "sqlx"))]
use sdf_data_structures::traits::SdfDataStructure;

use crate::error::SdfRepositoryError;

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd)]
pub(crate) struct SemanticVersion {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) patch: u16,
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl From<SemanticVersion> for Vec<u16> {
    fn from(val: SemanticVersion) -> Self {
        vec![val.major, val.minor, val.patch]
    }
}

impl From<SemanticVersion> for Vec<i32> {
    fn from(val: SemanticVersion) -> Self {
        vec![val.major.into(), val.minor.into(), val.patch.into()]
    }
}

impl From<SemanticVersion> for String {
    fn from(val: SemanticVersion) -> Self {
        format!("{}.{}.{}", val.major, val.minor, val.patch)
    }
}

impl TryFrom<Vec<u16>> for SemanticVersion {
    type Error = SdfRepositoryError;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        let mut iterator = value.into_iter();

        let major = iterator.next().ok_or(SdfRepositoryError::ModelQuery(
            "Invalid first sematic version component".to_string(),
        ))?;
        let minor = iterator.next().ok_or(SdfRepositoryError::ModelQuery(
            "Invalid second sematic version component".to_string(),
        ))?;
        let patch = iterator.next().ok_or(SdfRepositoryError::ModelQuery(
            "Invalid third sematic version component".to_string(),
        ))?;

        if iterator.next().is_some() {
            return Err(SdfRepositoryError::ModelQuery(
                "Unexpected fourth version element".to_string(),
            ));
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl TryFrom<String> for SemanticVersion {
    type Error = SdfRepositoryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split_versions = value.split(".");

        let version_numbers: Result<Vec<_>, _> = split_versions
            .into_iter()
            .map(|x| x.parse::<u16>())
            .collect();

        version_numbers
            .map_err(|x| SdfRepositoryError::ModelQuery(x.to_string()))?
            .try_into()
    }
}

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

        query_builder
    }

    #[cfg(not(feature = "sqlx"))]
    pub(crate) fn filter_model(self, sdf_model: &SdfModel) -> Result<bool, SdfRepositoryError> {
        let version = sdf_model.get_version().unwrap();

        let namespace = sdf_model.get_target_namespace().unwrap().unwrap();
        let lineage = sdf_model.get_lineage();

        if self.namespace != namespace && self.lineage != lineage {
            return Ok(false);
        }

        let parsed_semantic_version: Result<SemanticVersion, _> = version.try_into();
        let semantic_version = parsed_semantic_version.unwrap();

        if let Some(version) = &self.version
            && version != &semantic_version
        {
            return Ok(false);
        }

        if let Some(min_version) = &self.min_version
            && min_version > &semantic_version
        {
            return Ok(false);
        }

        if let Some(max_version) = &self.max_version
            && max_version < &semantic_version
        {
            return Ok(false);
        }

        if let Some(exclusive_min_version) = &self.exclusive_min_version
            && exclusive_min_version >= &semantic_version
        {
            return Ok(false);
        }

        if let Some(exclusive_max_version) = &self.exclusive_max_version
            && exclusive_max_version <= &semantic_version
        {
            return Ok(false);
        }

        Ok(true)
    }
}

impl TryFrom<&SdfSupplement> for QueryParameters {
    type Error = SdfRepositoryError;

    fn try_from(value: &SdfSupplement) -> Result<Self, Self::Error> {
        let version = if let Some(version) = value.get_target_version() {
            let semantic_version: SemanticVersion = version.try_into()?;

            Some(semantic_version)
        } else {
            None
        };

        let namespace = value
            .get_default_namespace_url()
            .ok_or(SdfRepositoryError::ModelQuery("TODO".to_string()))?;
        let lineage = value.get_lineage();

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

pub(crate) trait QueryHandler {
    async fn initialize(self) -> Result<(), SdfRepositoryError>;

    async fn delete_models(
        self,
        query: QueryParameters,
    ) -> Result<Vec<SdfModel>, SdfRepositoryError>;

    async fn get_model(&self, query: QueryParameters) -> Result<SdfModel, SdfRepositoryError>;

    async fn get_models(&self, query: QueryParameters)
    -> Result<Vec<SdfModel>, SdfRepositoryError>;

    async fn insert_model(&self, model: SdfModel) -> Result<SdfModel, SdfRepositoryError>;

    async fn update_model(
        &self,
        sdf_supplement: &SdfSupplement,
    ) -> Result<SdfModel, SdfRepositoryError>;
}
