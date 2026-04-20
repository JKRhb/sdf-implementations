// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use core::panic;

use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
use sqlx::{Error, QueryBuilder};

use crate::models::DatabaseRow;

#[derive(Debug)]
pub(crate) struct SemanticVersion {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) patch: u16,
}

impl Into<Vec<u16>> for SemanticVersion {
    fn into(self) -> Vec<u16> {
        vec![self.major, self.minor, self.patch]
    }
}

impl Into<Vec<i32>> for SemanticVersion {
    fn into(self) -> Vec<i32> {
        vec![self.major.into(), self.minor.into(), self.patch.into()]
    }
}

impl Into<String> for SemanticVersion {
    fn into(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl TryFrom<Vec<u16>> for SemanticVersion {
    type Error = Error;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        let mut iterator = value.into_iter();

        let major = iterator.next().unwrap();
        let minor = iterator.next().unwrap();
        let patch = iterator.next().unwrap();

        if iterator.next().is_some() {
            panic!()
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl TryFrom<String> for SemanticVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split_versions = value.split(".");

        let version_numbers: Result<Vec<_>, _> = split_versions
            .into_iter()
            .map(|x| x.parse::<u16>())
            .collect();

        version_numbers.unwrap().try_into()
    }
}

#[derive(Debug)]
pub(crate) struct QueryParameters {
    pub(crate) namespace: String,
    pub(crate) lineage: Option<String>,
    pub(crate) version: Option<SemanticVersion>,
    pub(crate) min_version: Option<SemanticVersion>,
    pub(crate) max_version: Option<SemanticVersion>,
    pub(crate) exclusive_min_version: Option<SemanticVersion>,
    pub(crate) exclusive_max_version: Option<SemanticVersion>,
}

impl QueryParameters {
    pub fn create_query_builder<'a>(self, init: impl Into<String>) -> QueryBuilder<'a, sqlx::Postgres> {
        let mut query_builder = QueryBuilder::new(init);

        query_builder.push(" WHERE namespace = ");
        query_builder.push_bind(self.namespace);

        query_builder.push(" AND lineage IS NOT DISTINCT FROM ");
        query_builder.push_bind(self.lineage);

        for (comparator, semantic_version) in vec![
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
        // query_builder.build_query_as::<DatabaseRow>()
    }
}

impl TryFrom<&SdfSupplement> for QueryParameters {
    type Error = Error;

    fn try_from(value: &SdfSupplement) -> Result<Self, Self::Error> {
        let version = value.get_target_version().map(|x| x.try_into().unwrap());
        let namespace = value.get_default_namespace_url().unwrap();
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
    async fn initialize(self) -> Result<(), Error>;

    async fn delete_models(self, query: QueryParameters) -> Result<Vec<SdfModel>, Error>;

    async fn get_model(&self, query: QueryParameters) -> Result<SdfModel, Error>;

    async fn get_models(self, query: QueryParameters) -> Result<Vec<SdfModel>, Error>;

    async fn insert_model(&self, model: SdfModel) -> Result<SdfModel, Error>;

    async fn update_model(&self, sdf_supplement: &SdfSupplement) -> Result<SdfModel, Error>;
}
