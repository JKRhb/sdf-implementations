// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};

use anyhow::bail;
use async_trait::async_trait;
use reqwest::{Method, Url};
use sdf_data_structures::{
    instance::{
        InfoBlockBuilder, SdfInstanceBuilder, SdfInstanceOfBuilder, SdfMessage, SdfMessageBuilder,
    },
    model::{
        SdfModel,
        affordances::{sdf_action::SdfAction, sdf_property::SdfProperty},
        protocol_mappings::{coap::CoapAction, http::HttpProperty},
    },
};
use serde_json::Value;
use uuid::Uuid;

use crate::{cli::ObserveHandler, protocols::ProtocolImplementation};

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

    async fn perform_configuration(
        self,
        instance_url: Url,
        sdf_snapshot: SdfMessage,
        input_value: HashMap<String, Value>,
    ) -> anyhow::Result<()> {
        // TODO: Refactor code for generating configuration messages from snapshots
        let sdf_message = SdfMessageBuilder::default()
            .info(
                InfoBlockBuilder::default()
                    .message_id(Uuid::new_v4())
                    .build()?,
            )
            .sdf_instance_of(
                SdfInstanceOfBuilder::default()
                    .entry_point(sdf_snapshot.get_entry_point())
                    .build()?,
            )
            .sdf_instance(
                SdfInstanceBuilder::default()
                    .sdf_context(input_value)
                    .build()?,
            )
            .build()?;

        let payload = serde_json::to_string(&sdf_message)?;

        let request = self
            .client
            .request(Method::POST, instance_url)
            .body(payload)
            .build()?;

        self.client.execute(request).await?.json::<Value>().await?;

        Ok(())
    }

    async fn perform_read_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
    ) -> anyhow::Result<Value> {
        let property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, sdf_model)?
            .clone();

        let url: Url = property
            .read_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = property.read_method().as_str().try_into()?;

        let request = self.client.request(method, url).build()?;

        let result = self.client.execute(request).await?.json::<Value>().await?;

        Ok(result)
    }

    async fn perform_observe_operation(
        &self,
        _scheme: &str,
        _sdf_message: SdfMessage,
        _sdf_model: &SdfModel,
        _property_pointer: String,
        _observe_handler: ObserveHandler,
    ) -> anyhow::Result<()> {
        bail!("Observe operations are not yet supported by HTTP(S).")
    }

    async fn perform_write_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
        input_value: Value,
    ) -> anyhow::Result<()> {
        let property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, sdf_model)?
            .clone();

        let url: Url = property
            .read_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = property.read_method().as_str().try_into()?;

        let request = self
            .client
            .request(method, url)
            .body(serde_json::to_string(&input_value)?)
            .build()?;

        self.client.execute(request).await?.error_for_status()?;

        Ok(())
    }

    async fn perform_invoke_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        action_pointer: String,
        input_value: Option<Value>,
    ) -> anyhow::Result<Option<Value>> {
        let sdf_action = sdf_message
            .resolve_pointer_against_model::<&SdfAction>(&action_pointer, sdf_model)?
            .clone();

        let url: Url = sdf_action
            .invoke_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = sdf_action.invoke_method().as_str().try_into()?;

        let request = self
            .client
            .request(method, url)
            .json(&input_value)
            .build()?;

        let result = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .json::<Option<Value>>()
            .await?;

        Ok(result)
    }

    async fn perform_subscribe_operation(
        &self,
        _scheme: &str,
        _sdf_message: SdfMessage,
        _sdf_model: &SdfModel,
        _property_pointer: String,
        _observe_handler: ObserveHandler,
    ) -> anyhow::Result<()> {
        bail!("Subscribe operations are not yet supported by HTTP(S).")
    }

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        let sdf_instance = reqwest::get(instance_url)
            .await?
            .json::<SdfMessage>()
            .await?;

        return Ok(sdf_instance);
    }
}
