use std::sync::Arc;

use crate::{
    SdfConsumerError,
    protocol_mappings::{Operation, SupportedProtocols},
};
use clap::Parser;
use coap::{
    UdpCoAPClient,
    client::CoAPClient,
    dtls::UdpDtlsConfig,
    request::{Method, RequestBuilder},
};
use serde_json::Value;
use std::net::ToSocketAddrs;
use thiserror::Error;
use webrtc_dtls::{cipher_suite::CipherSuiteId, config::Config};

/// Domain-specific errors
#[derive(Error, Debug)]
pub(crate) enum CliError {
    #[error("Please specify one of the available subcommands!")]
    MissingCommand(),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// The operation that is supposed to be performed with the affordance.
    ///
    /// Only has to be provided for properties at the moment if the user
    /// intends to write a property instead of reading it.
    #[command(subcommand)]
    pub(crate) operation: Operation,

    /// URL pointing to a resource retrieving instance-related messages.
    pub(crate) instance_url: String,

    /// JSON Pointer to the affordance that is to be used.
    ///
    /// The JSON Pointer must match the path within the SDF model, not the
    /// instance.
    pub(crate) affordance_pointer: String,

    /// Preferred protocol map for interactions.
    ///
    /// If unset, coap will be used by default if present in the resolved
    /// model.
    preferred_protocol: Option<SupportedProtocols>,
}

impl Cli {
    // TODO: Maybe refactor this function
    pub(crate) fn get_protocol_preference(&self) -> Vec<SupportedProtocols> {
        let preferred_protocol = self.preferred_protocol.unwrap_or(SupportedProtocols::Coap);

        let mut protocol_order = vec![preferred_protocol];

        for protocol in [SupportedProtocols::Coap, SupportedProtocols::Http] {
            if protocol_order.contains(&protocol) {
                continue;
            }

            protocol_order.push(protocol);
        }

        protocol_order
    }

    pub(crate) async fn obtain_sdf_instance(&self) -> anyhow::Result<Value> {
        let instance_url = &self.instance_url;

        if instance_url.starts_with("http") {
            let sdf_instance = reqwest::get(instance_url).await?.json::<Value>().await?;

            return Ok(sdf_instance);
        } else if instance_url.starts_with("coaps") {
            let config = Config {
                psk: Some(Arc::new(|_| Ok("secretPSK".as_bytes().to_vec()))),
                cipher_suites: vec![CipherSuiteId::Tls_Psk_With_Aes_128_Ccm_8],
                psk_identity_hint: Some("identity".as_bytes().to_vec()),
                ..Default::default()
            };

            let dtls_config = UdpDtlsConfig {
                config,
                dest_addr: ("192.168.178.45", 5684)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap(),
            };

            let client = CoAPClient::from_udp_dtls_config(dtls_config)
                .await
                .expect("could not create client");
            let domain = "192.168.178.45:5684";

            let request = RequestBuilder::new("/.well-known/sdf/instance", Method::Get)
                .domain(domain.to_string())
                .build();

            let response = client.send(request).await.unwrap();
            let payload_string = String::from_utf8(response.message.payload).unwrap();

            let sdf_instance = serde_json::from_str(&payload_string)?;

            println!("{sdf_instance}");

            return Ok(sdf_instance);
        } else if instance_url.starts_with("coap") {
            let response = UdpCoAPClient::get(instance_url).await.unwrap();
            let payload_string = String::from_utf8(response.message.payload).unwrap();

            let sdf_instance = serde_json::from_str(&payload_string)?;

            return Ok(sdf_instance);
        }

        Err(SdfConsumerError {
            error_message: "Unsupported URI scheme!".to_string(),
        }
        .into())
    }
}
