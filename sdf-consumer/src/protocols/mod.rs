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
use sdf_data_structures::{
    instance::SdfMessage,
    traits::{SdfAffordance, SdfGrouping},
};
use serde_json::Value;

use crate::{
    consumer::ConsumedSdfProperty,
    error::SdfConsumerError,
    protocols::{coap::CoapImplementation, coaps::CoapsImplementation, http::HttpImplementation},
};

pub(crate) mod coap;
pub(crate) mod coaps;
pub(super) mod common;
pub(crate) mod http;

impl TryFrom<Url> for Box<dyn ProtocolImplementation> {
    type Error = SdfConsumerError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match value.scheme() {
            "coap" => Ok(Box::from(CoapImplementation {})),
            "coaps" => Ok(Box::from(CoapsImplementation {})),
            "http" | "https" => Ok(Box::from(HttpImplementation {})),
            _ => Err(SdfConsumerError {
                error_message: "hi".to_string(),
            }),
        }
    }
}

#[async_trait]
pub trait ProtocolImplementation {
    fn supported_uri_schemes(&self) -> HashSet<String>;

    async fn perform_configuration(&self) -> anyhow::Result<()>;

    async fn perform_read_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<Value>;

    async fn perform_observe_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<()>;

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage>;
}
