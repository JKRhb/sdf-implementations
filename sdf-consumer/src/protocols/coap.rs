// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};
use std::net::ToSocketAddrs;
use std::sync::Arc;

use anyhow::{Context, bail};
use async_trait::async_trait;
use coap::UdpCoAPClient;
use coap::client::{CoAPClient, ObserveMessage};
use coap::dtls::{DtlsConnection, UdpDtlsConfig};
use coap::request::{CoapOption, MessageClass, Method, Packet};
use coap_lite::ContentFormat;
use reqwest::Url;
use sdf_data_structures::constants::{
    SDF_PATCH_MESSAGE_CONTENT_FORMAT, SDF_SNAPSHOT_MESSAGE_CONTENT_FORMAT,
};
use sdf_data_structures::instance::{
    InfoBlockBuilder, SdfInstanceBuilder, SdfInstanceOfBuilder, SdfMessage, SdfMessageBuilder,
};
use sdf_data_structures::model::SdfModel;
use sdf_data_structures::model::affordances::sdf_action::SdfAction;
use sdf_data_structures::model::affordances::sdf_event::SdfEvent;
use sdf_data_structures::model::affordances::sdf_property::SdfProperty;
use sdf_data_structures::model::protocol_mappings::coap::{CoapAction, CoapEvent, CoapProperty};
use serde_json::Value;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;
use webrtc_dtls::cipher_suite::CipherSuiteId;

use crate::protocols::ProtocolImplementation;

fn map_string_to_coap_method(method: String) -> Method {
    match method.as_str() {
        "GET" => Method::Get,
        "PATCH" => Method::Patch,
        "POST" => Method::Post,
        "DELETE" => Method::Delete,
        "iPATCH" => Method::IPatch,
        "FETCH" => Method::Fetch,
        _ => Method::UnKnown,
    }
}

enum InternalCoapClient {
    Udp(coap::client::UdpCoAPClient),
    Dtls(coap::client::CoAPClient<DtlsConnection>),
}

pub(crate) type ObserveHandler = Box<dyn FnMut(Packet) + Send + 'static>;

impl InternalCoapClient {
    async fn send_request(
        self,
        url: &Url,
        method: Method,
        payload_value: Option<Value>,
        content_format: Option<u16>,
        accept: Option<u16>,
    ) -> anyhow::Result<Option<Value>> {
        let mut request = coap::request::CoapRequest::new();

        request.set_method(method);
        request.set_path(url.path());

        if let Some(accept) = accept {
            request
                .message
                .add_option(CoapOption::Accept, accept.to_be_bytes().into());
        }

        if let Some(content_format) = content_format {
            request.message.add_option(
                CoapOption::ContentFormat,
                content_format.to_be_bytes().into(),
            );
        }

        if let Some(payload_value) = payload_value {
            let payload = serde_json::to_vec(&payload_value)?;
            request.message.payload = payload;
        }

        let response = match self {
            InternalCoapClient::Udp(udp_client) => udp_client.send(request).await?,
            InternalCoapClient::Dtls(dtls_client) => dtls_client.send(request).await?,
        };

        let response_code = response.get_status();

        if response_code.is_error() {
            bail!(
                "Request failed with response code {}",
                MessageClass::Response(*response_code)
            )
        }

        let response_payload = response.message.payload;

        if response_payload.is_empty() {
            return Ok(None);
        }

        let value = serde_json::from_slice::<Value>(&response_payload)?;

        Ok(Some(value))
    }

    async fn observe(
        self,
        method: Method,
        accept: Option<u16>,
        observe_handler: ObserveHandler,
    ) -> anyhow::Result<Sender<ObserveMessage>> {
        let mut request = coap::request::CoapRequest::new();

        request.set_method(method);

        if let Some(accept) = accept {
            request
                .message
                .add_option(CoapOption::Accept, accept.to_be_bytes().into());
        }

        let response = match self {
            InternalCoapClient::Udp(udp_client) => {
                udp_client.observe_with(request, observe_handler).await?
            }
            InternalCoapClient::Dtls(dtls_client) => {
                dtls_client.observe_with(request, observe_handler).await?
            }
        };

        Ok(response)
    }
}

#[derive(Default)]
pub(crate) struct CoapImplementation {
    dtls_psk_callback: Option<PskCallback>,
    dtls_psk_identity_hint: Option<String>,
}

pub(crate) type PskCallback =
    Arc<dyn (Fn(&[u8]) -> std::result::Result<Vec<u8>, webrtc_dtls::Error>) + Send + Sync>;

impl CoapImplementation {
    pub(crate) fn new(
        dtls_psk_callback: Option<PskCallback>,
        dtls_psk_identity_hint: Option<String>,
    ) -> Self {
        CoapImplementation {
            dtls_psk_callback,
            dtls_psk_identity_hint,
        }
    }

    fn obtain_addr(url: &Url) -> anyhow::Result<(String, u16)> {
        let scheme = url.scheme();

        let default_port = match scheme {
            "coap" => 5683,
            "coaps" => 5684,
            _ => bail!("Unsupported URI scheme!"),
        };

        let port = url.port().unwrap_or(default_port);

        let host = url.host().context("Missing host component")?.to_string();

        Ok((host, port))
    }

    fn generate_dlts_config(
        cipher_suites: Vec<CipherSuiteId>,
        psk: Option<PskCallback>,
        psk_identity_hint: Option<Vec<u8>>,
    ) -> webrtc_dtls::config::Config {
        webrtc_dtls::config::Config {
            psk,
            psk_identity_hint,
            cipher_suites,
            ..Default::default()
        }
    }

    fn create_internal_observe_handler(
        mut cli_observe_handler: crate::cli::ObserveHandler,
    ) -> ObserveHandler {
        Box::new(move |packet| {
            let value = serde_json::to_value(packet.payload);

            let result: anyhow::Result<Value> = match value {
                Ok(value) => anyhow::Ok(value),
                Err(error) => anyhow::Result::Err(anyhow::anyhow!(error)),
            };

            cli_observe_handler(result);
        })
    }

    async fn obtain_client(&self, url: &Url) -> anyhow::Result<InternalCoapClient> {
        let uri_scheme = url.scheme();
        let addr = Self::obtain_addr(url)?;
        match uri_scheme {
            "coap" => {
                let client = UdpCoAPClient::new(addr).await?;

                Ok(InternalCoapClient::Udp(client))
            }
            "coaps" => {
                let psk_identity_hint = self
                    .dtls_psk_identity_hint
                    .as_ref()
                    .map(|x| x.as_bytes().into());

                let psk = self.dtls_psk_callback.clone();

                let udp_dtls_config =
                    Self::generate_dlts_config(all_cipher_suites(), psk, psk_identity_hint);

                let dest_addr = addr
                    .to_socket_addrs()?
                    .next()
                    .context("Name resolution did not return any suitable IP address.")?;

                let config = UdpDtlsConfig {
                    config: udp_dtls_config,
                    dest_addr,
                };

                let client = CoAPClient::from_udp_dtls_config(config).await?;

                Ok(InternalCoapClient::Dtls(client))
            }

            _ => bail!(format!("Unsupported URI scheme {uri_scheme}")),
        }
    }
}

#[async_trait]
impl ProtocolImplementation for CoapImplementation {
    fn supported_uri_schemes(&self) -> HashSet<&'static str> {
        HashSet::from(["coap", "coaps"])
    }

    async fn perform_configuration(
        self,
        instance_url: Url,
        sdf_snapshot: SdfMessage,
        input_value: HashMap<String, Value>,
    ) -> anyhow::Result<()> {
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

        let payload = serde_json::to_value(sdf_message)?;

        let client = self.obtain_client(&instance_url).await?;

        client
            .send_request(
                &instance_url,
                Method::Post,
                Some(payload),
                None,
                Some(SDF_PATCH_MESSAGE_CONTENT_FORMAT),
            )
            .await?;

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
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, sdf_model)?;

        let url: Url = property
            .read_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = property.read_method();
        let method = map_string_to_coap_method(method);

        let client = self.obtain_client(&url).await?;

        // TODO: Replace hardcoded values
        let accept: usize = ContentFormat::ApplicationJSON.into();
        let accept: u16 = accept.try_into().unwrap();

        client
            .send_request(&url, method, None, None, Some(accept))
            .await?
            .context("Read operation did not return a value")
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
            .write_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = property.write_method();
        let method = map_string_to_coap_method(method);

        let client = self.obtain_client(&url).await?;

        // TODO: Replace hardcoded values
        let content_format: usize = ContentFormat::ApplicationJSON.into();
        let content_format: u16 = content_format.try_into().unwrap();

        client
            .send_request(&url, method, Some(input_value), Some(content_format), None)
            .await?;

        Ok(())
    }

    async fn perform_observe_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
        observe_handler: crate::cli::ObserveHandler,
    ) -> anyhow::Result<()> {
        let property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, sdf_model)?
            .clone();

        let url: Url = property
            .read_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = property.read_method();
        let method = map_string_to_coap_method(method);

        let client = self.obtain_client(&url).await?;

        // TODO: Replace hardcoded values
        let accept: usize = ContentFormat::ApplicationJSON.into();
        let accept: u16 = accept.try_into().unwrap();

        let internal_observe_handler = Self::create_internal_observe_handler(observe_handler);

        client
            .observe(method, Some(accept), internal_observe_handler)
            .await?;

        Ok(())
    }

    async fn perform_subscribe_operation(
        &self,
        scheme: &str,
        sdf_message: SdfMessage,
        sdf_model: &SdfModel,
        property_pointer: String,
        observe_handler: crate::cli::ObserveHandler,
    ) -> anyhow::Result<()> {
        let event = sdf_message
            .resolve_pointer_against_model::<&SdfEvent>(&property_pointer, sdf_model)?
            .clone();

        let url: Url = event
            .subscribe_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = event.suscribe_method();
        let method = map_string_to_coap_method(method);

        let client = self.obtain_client(&url).await?;

        // TODO: Replace hardcoded values
        let accept: usize = ContentFormat::ApplicationJSON.into();
        let accept: u16 = accept.try_into().unwrap();

        let internal_observe_handler = Self::create_internal_observe_handler(observe_handler);

        client
            .observe(method, Some(accept), internal_observe_handler)
            .await?;

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
        let action = sdf_message
            .resolve_pointer_against_model::<&SdfAction>(&action_pointer, sdf_model)?
            .clone();

        let url: Url = action
            .invoke_url(scheme, sdf_model, &sdf_message)?
            .as_str()
            .try_into()?;

        let method = action.invoke_method();
        let method = map_string_to_coap_method(method);

        let client = self.obtain_client(&url).await?;

        // TODO: Replace hardcoded values
        let content_format: usize = ContentFormat::ApplicationJSON.into();
        let content_format: u16 = content_format.try_into().unwrap();
        let accept = content_format;

        client
            .send_request(
                &url,
                method,
                input_value,
                Some(content_format),
                Some(accept),
            )
            .await
    }

    async fn obtain_sdf_snapshot(&self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        let client = self.obtain_client(&instance_url).await?;

        let payload_value = client
            .send_request(
                &instance_url,
                Method::Get,
                None,
                None,
                Some(SDF_SNAPSHOT_MESSAGE_CONTENT_FORMAT),
            )
            .await?
            .context("Request did not return any payload")?;

        let sdf_snapshot = serde_json::from_value::<SdfMessage>(payload_value)?;

        Ok(sdf_snapshot)
    }
}

fn all_cipher_suites() -> Vec<CipherSuiteId> {
    vec![
        CipherSuiteId::Tls_Ecdhe_Ecdsa_With_Aes_128_Ccm,
        CipherSuiteId::Tls_Ecdhe_Ecdsa_With_Aes_128_Ccm_8,
        CipherSuiteId::Tls_Ecdhe_Ecdsa_With_Aes_128_Gcm_Sha256,
        CipherSuiteId::Tls_Ecdhe_Rsa_With_Aes_128_Gcm_Sha256,
        CipherSuiteId::Tls_Ecdhe_Ecdsa_With_Aes_256_Cbc_Sha,
        CipherSuiteId::Tls_Ecdhe_Rsa_With_Aes_256_Cbc_Sha,
        CipherSuiteId::Tls_Psk_With_Aes_128_Ccm,
        CipherSuiteId::Tls_Psk_With_Aes_128_Ccm_8,
        CipherSuiteId::Tls_Psk_With_Aes_128_Gcm_Sha256,
    ]
}
