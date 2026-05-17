// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, str::FromStr};

use anyhow::{Context, bail};
use async_trait::async_trait;
use reqwest::Url;
use sdf_data_structures::{
    instance::SdfMessage, model::protocol_mappings::http::PropertyHttpProtocolMap,
};
use serde_json::Value;

use crate::{consumer::ConsumedSdfProperty, protocols::ProtocolImplementation};

trait HttpProtocolMapping {
    fn obtain_protocol_map(&self) -> Option<PropertyHttpProtocolMap>;

    fn url(&self) -> Option<String>;

    fn scheme(&self) -> String;

    fn port(&self) -> u16;

    fn authority(&self) -> Option<String>;

    fn method(&self) -> String;
}

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
        let href = href.trim_start_matches("/");

        Some(format!("{scheme}:{authority}/{href}"))
    }

    fn method(&self) -> String {
        let sdf_protocol_map = self.obtain_protocol_map();

        sdf_protocol_map
            .and_then(|x| x.sdf_operations)
            .and_then(|x| x.read)
            .map(|x| x.method)
            .unwrap_or("GET".to_string())
    }

    fn scheme(&self) -> String {
        "http".to_string()
    }

    fn authority(&self) -> Option<String> {
        Some("httpbin.org".to_string())
    }

    fn port(&self) -> u16 {
        80
    }
}

pub(crate) struct HttpImplementation {
    client: reqwest::Client,
}

impl HttpImplementation {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ProtocolImplementation for HttpImplementation {
    fn supported_uri_schemes(&self) -> HashSet<&'static str> {
        HashSet::from(["http", "https"])
    }

    async fn perform_configuration(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn perform_read_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<Value> {
        let url = consumed_sdf_property
            .url()
            .context("Error constructing HTTP URI")?;

        let method: reqwest::Method = consumed_sdf_property.method().as_str().try_into()?;

        let request = self.client.request(method, url).build()?;

        let result = self.client.execute(request).await?.json::<Value>().await?;

        Ok(result)
    }

    async fn perform_observe_operation(
        &self,
        _consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn perform_write_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
        _input_value: Value,
    ) -> anyhow::Result<()> {
        let url = consumed_sdf_property
            .url()
            .context("Error constructing HTTP URI")?;

        let method: reqwest::Method = consumed_sdf_property.method().as_str().try_into()?;

        let request = self
            .client
            .request(method, url)
            .body(serde_json::to_string(&_input_value)?)
            .build()?;

        self.client.execute(request).await?.error_for_status()?;

        Ok(())
    }

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        let sdf_instance = reqwest::get(instance_url)
            .await?
            .json::<SdfMessage>()
            .await?;

        return Ok(sdf_instance);
    }
}
