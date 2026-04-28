// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};

use crate::{error::SdfRepositoryError, models::query_parameters::QueryParameters};

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
