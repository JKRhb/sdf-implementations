// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, fmt::format};

use anyhow::{Context, bail};
use async_trait::async_trait;
use reqwest::Url;
use sdf_data_structures::{
    instance::SdfMessage,
    model::protocol_mappings::http::{HttpProtocolMap, PropertyHttpProtocolMap},
};
use serde_json::{Map, Value};

use crate::{
    consumer::ConsumedSdfProperty,
    protocols::{
        ProtocolImplementation,
        common::{determine_url, obtain_method, obtain_operation},
    },
};

trait HttpProtocolMapping {
    fn obtain_protocol_map(&self) -> Option<PropertyHttpProtocolMap>;

    fn url(&self) -> Option<String>;

    fn scheme(&self) -> String;

    fn port(&self) -> u16;

    fn authority(&self) -> Option<String>;

    fn method(&self) -> String;
}

trait HttpPropertyProtocolMapping {}

impl HttpProtocolMapping for ConsumedSdfProperty {
    fn obtain_protocol_map(&self) -> Option<PropertyHttpProtocolMap> {
        self.internal_data.clone().sdf_protocol_map?.http
    }

    fn url(&self) -> Option<String> {
        let sdf_protocol_map = self.obtain_protocol_map()?;

        let port = self.port();

        let scheme = self.scheme();
        let authority = self
            .authority()
            .map(|x| format!("//{x}"))
            .map(|x| format!("{x}:{port}"))
            .unwrap_or_default();
        let href = sdf_protocol_map.sdf_operations?.read.unwrap().href;

        Some(format!("{scheme}{authority}/{href}"))
    }

    fn method(&self) -> String {
        let sdf_protocol_map = self.obtain_protocol_map();

        sdf_protocol_map
            .and_then(|x| x.sdf_operations)
            .and_then(|x| x.read)
            .and_then(|x| Some(x.method))
            .unwrap_or("GET".to_string())
    }

    fn scheme(&self) -> String {
        "http".to_string()
    }

    fn authority(&self) -> Option<String> {
        Some("example.org".to_string())
    }

    fn port(&self) -> u16 {
        80
    }
}

pub(crate) struct HttpImplementation {}

#[async_trait]
impl ProtocolImplementation for HttpImplementation {
    fn supported_uri_schemes(&self) -> HashSet<String> {
        HashSet::from(["http".to_string(), "https".to_string()])
    }

    async fn perform_configuration(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn perform_read_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<Value> {
        let url = consumed_sdf_property.url().context("hey")?;
        let method = consumed_sdf_property.method();

        match method.as_str() {
            "GET" => {
                let result = reqwest::get(url).await?.json::<Value>().await?;

                Ok(result)
            }
            _ => bail!("Unknown Method name"),
        }
    }

    async fn perform_observe_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        let sdf_instance = reqwest::get(instance_url)
            .await?
            .json::<SdfMessage>()
            .await?;

        return Ok(sdf_instance);
    }
}

impl HttpImplementation {
    pub(crate) async fn perform_write_operation(
        self,
        http_protocol_map: &Map<String, Value>,
        sdf_model: &Value,
        sdf_instance: &Value,
        input: &Value,
    ) -> anyhow::Result<Option<Value>> {
        let write_operation = obtain_operation(http_protocol_map, "write".to_string())?;

        let url = determine_url(
            write_operation,
            http_protocol_map,
            sdf_instance,
            sdf_model,
            "http",
        )?;

        let method = obtain_method(write_operation, "PUT");

        match method.as_str() {
            "PUT" => {
                reqwest::Client::new()
                    .put(url)
                    .body(serde_json::to_string(input)?)
                    .send()
                    .await?;

                Ok(None)
            }
            // TODO: Handle other methods as well
            _ => Ok(None),
        }
    }

    pub(crate) fn new() -> Self {
        Self {}
    }
}
