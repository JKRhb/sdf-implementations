// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use reqwest::Url;
use sdf_data_structures::{instance::SdfMessage, model::SdfModel};
use serde_json::Value;

use crate::{
    cli::{ObserveHandler, SecurityArguments},
    error::SdfConsumerError,
    protocols::{
        coap::{CoapImplementation, PskCallback},
        http::HttpImplementation,
    },
};

pub(crate) mod coap;
pub(crate) mod http;

impl TryFrom<(Url, SecurityArguments)> for Box<dyn ProtocolImplementation> {
    type Error = SdfConsumerError;

    fn try_from(value: (Url, SecurityArguments)) -> Result<Self, Self::Error> {
        let (url, security_arguments) = value;

        let psk_callback: Option<PskCallback> = (&security_arguments).into();

        match url.scheme() {
            "coap" | "coaps" => Ok(Box::from(CoapImplementation::new(
                psk_callback,
                security_arguments.dtls_identity,
            ))),
            "http" | "https" => Ok(Box::from(HttpImplementation::new())),
            _ => Err(SdfConsumerError {
                error_message: "hi".to_string(),
            }),
        }
    }
}

#[async_trait]
pub trait ProtocolImplementation {
    fn supported_uri_schemes(&self) -> HashSet<&'static str>;

    async fn perform_configuration(
        self,
        instance_url: Url,
        sdf_snapshot: SdfMessage,
        input_value: HashMap<String, Value>,
    ) -> anyhow::Result<()>;

    async fn perform_read_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
    ) -> anyhow::Result<Value>;

    async fn perform_observe_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
        observe_handler: ObserveHandler,
    ) -> anyhow::Result<()>;

    async fn perform_write_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
        input_value: Value,
    ) -> anyhow::Result<()>;

    async fn perform_invoke_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        action_pointer: String,
        input_value: Option<Value>,
    ) -> anyhow::Result<Option<Value>>;

    async fn perform_subscribe_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        event_pointer: String,
        observe_handler: ObserveHandler,
    ) -> anyhow::Result<()>;

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage>;
}
