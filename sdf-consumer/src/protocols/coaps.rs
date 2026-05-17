// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use async_trait::async_trait;
use reqwest::Url;
use sdf_data_structures::instance::SdfMessage;
use serde_json::Value;

use crate::{consumer::ConsumedSdfProperty, protocols::ProtocolImplementation};

pub(crate) struct CoapsImplementation {}

#[async_trait]
impl ProtocolImplementation for CoapsImplementation {
    fn supported_uri_schemes(&self) -> HashSet<&'static str> {
        HashSet::from(["coaps"])
    }

    async fn perform_configuration(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn perform_read_operation(
        &self,
        _consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<Value> {
        todo!()
    }

    async fn perform_write_operation(
        &self,
        _consumed_sdf_property: ConsumedSdfProperty,
        _input_value: Value,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn perform_observe_operation(
        &self,
        _consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn obtain_sdf_snapshot(&self, _instance_url: Url) -> anyhow::Result<SdfMessage> {
        todo!()
    }
}
