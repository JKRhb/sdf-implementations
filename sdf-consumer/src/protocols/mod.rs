// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use clap::ValueEnum;
use reqwest::Url;
use sdf_data_structures::{
    instance::SdfMessage,
    traits::{SdfAffordance, SdfGrouping},
};
use serde_json::Value;

use crate::{
    error::SdfConsumerError,
    protocols::{coap::CoapImplementation, coaps::CoapsImplementation, http::HttpImplementation},
};

pub(crate) mod coap;
pub(crate) mod coaps;
pub(super) mod common;
pub(crate) mod http;

#[derive(Clone, Copy, ValueEnum, PartialEq, Debug)]
pub(crate) enum SupportedProtocols {
    Coap,
    Coaps,
    Http,
    Https,
}

pub(crate) enum ProtocolImplementation {
    Coap(CoapImplementation),
    Coaps(CoapsImplementation),
    Http(HttpImplementation),
}

impl ProtocolImplementation {
    pub(crate) fn try_new(
        interaction_affordance: SdfAffordance,
        sdf_grouping: SdfGrouping,
        preferred_protocol: Option<SupportedProtocols>,
    ) -> anyhow::Result<Self> {
        Ok(ProtocolImplementation::Http(HttpImplementation {}))
        // let blah = Blah {
        //     interaction_affordance,
        //     sdf_grouping,
        //     preferred_protocol,
        // };
    }
}

impl TryFrom<Url> for ProtocolImplementation {
    type Error = SdfConsumerError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match value.scheme() {
            "coap" => Ok(ProtocolImplementation::Coap(CoapImplementation {})),
            "coaps" => Ok(ProtocolImplementation::Coaps(CoapsImplementation {})),
            "http" | "https" => Ok(ProtocolImplementation::Http(HttpImplementation {})),
            _ => Err(SdfConsumerError {
                error_message: "hi".to_string(),
            }),
        }
    }
}

// TODO: Maybe needs better name
impl ProtocolImplementation {
    fn supported_uri_schemes() -> Vec<String> {
        todo!()
    }

    pub(crate) async fn obtain_sdf_snapshot(self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        match self {
            ProtocolImplementation::Coap(coap_protocol_mapping) => todo!(),
            ProtocolImplementation::Coaps(coaps_protocol_mapping) => todo!(),
            ProtocolImplementation::Http(http_protocol_mapping) => {
                http_protocol_mapping
                    .obtain_sdf_instance(instance_url)
                    .await
            }
        }
    }

    pub(crate) async fn perform_read_operation(
        self,
        url: String,
        // protocol_map: &Map<String, Value>,
        // sdf_model: &Value,
        // sdf_instance: &Value,
    ) -> anyhow::Result<Option<Value>> {
        match self {
            ProtocolImplementation::Http(http_protocol_mapping) => {
                http_protocol_mapping.perform_read_operation(url).await
            }
            ProtocolImplementation::Coap(coap_protocol_mapping) => todo!(),
            ProtocolImplementation::Coaps(coaps_protocol_mapping) => todo!(),
        }
    }

    pub(crate) async fn perform_observe_operation(self) -> anyhow::Result<()> {
        { Ok(()) }
    }
}
