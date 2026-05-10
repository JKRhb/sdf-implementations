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
    protocols::{
        coap::CoapProtocolMapping, coaps::CoapsProtocolMapping, http::HttpProtocolMapping,
    },
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

pub(crate) enum ProtocolMapping {
    CoapProtocolMapping(CoapProtocolMapping),
    CoapsProtocolMapping(CoapsProtocolMapping),
    HttpProtocolMapping(HttpProtocolMapping),
}

impl ProtocolMapping {
    pub(crate) fn try_new(
        interaction_affordance: SdfAffordance,
        sdf_grouping: SdfGrouping,
        preferred_protocol: Option<SupportedProtocols>,
    ) -> anyhow::Result<Self> {
        Ok(ProtocolMapping::HttpProtocolMapping(HttpProtocolMapping {}))
        // let blah = Blah {
        //     interaction_affordance,
        //     sdf_grouping,
        //     preferred_protocol,
        // };
    }
}

impl TryFrom<Url> for ProtocolMapping {
    type Error = SdfConsumerError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match value.scheme() {
            "coap" => Ok(ProtocolMapping::CoapProtocolMapping(CoapProtocolMapping {})),
            "coaps" => Ok(ProtocolMapping::CoapsProtocolMapping(
                CoapsProtocolMapping {},
            )),
            "http" | "https" => Ok(ProtocolMapping::HttpProtocolMapping(HttpProtocolMapping {})),
            _ => Err(SdfConsumerError {
                error_message: "hi".to_string(),
            }),
        }
    }
}

// TODO: Maybe needs better name
impl ProtocolMapping {
    fn supported_uri_schemes() -> Vec<String> {
        todo!()
    }

    pub(crate) async fn obtain_sdf_snapshot(self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        match self {
            ProtocolMapping::CoapProtocolMapping(coap_protocol_mapping) => todo!(),
            ProtocolMapping::CoapsProtocolMapping(coaps_protocol_mapping) => todo!(),
            ProtocolMapping::HttpProtocolMapping(http_protocol_mapping) => {
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
            ProtocolMapping::HttpProtocolMapping(http_protocol_mapping) => {
                http_protocol_mapping.perform_read_operation(url).await
            }
            ProtocolMapping::CoapProtocolMapping(coap_protocol_mapping) => todo!(),
            ProtocolMapping::CoapsProtocolMapping(coaps_protocol_mapping) => todo!(),
        }
    }
}
