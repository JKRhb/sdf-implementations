// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use anyhow::Context;
use clap::Subcommand;
use reqwest::Url;
use sdf_data_structures::{model::SdfModel, traits::SdfGrouping};
use serde_json::Value;

use crate::protocols::{ProtocolMapping, SupportedProtocols};

#[derive(Subcommand)]
pub(crate) enum Operation {
    /// Reads a property from an SDF Thing
    Read {
        #[clap(long, short)]
        observe: bool,
    },

    /// Writes the property of an SDF Thing
    Write { input: Option<Value> },

    /// Invokes an action of an SDF Thing.
    Invoke,

    /// Subscribes to an event of an SDF Thing.
    Subscribe,

    /// Reconfigures a Thing
    Configure { input_file_name: String },

    ListConfigParameters {
        #[clap(long, short)]
        show_schema: bool,
    },
}

impl Operation {
    pub(crate) async fn handle_operation(
        self,
        instance_url: Url,
        preferred_protocol: Option<SupportedProtocols>,
        affordance_pointer: String,
    ) -> anyhow::Result<()> {
        let protocol_mapping: ProtocolMapping = instance_url.clone().try_into()?;

        let sdf_snapshot = protocol_mapping.obtain_sdf_snapshot(instance_url).await?;

        let model_url = sdf_snapshot.get_sdf_model_url()?.context("hi")?;

        let sdf_model = reqwest::get(model_url).await?.json::<SdfModel>().await?;

        // TODO: Handle pointer prefix
        let sdf_grouping = sdf_model.resolve_entry_point_from_sdf_message(sdf_snapshot)?;

        let interaction_affordance = sdf_grouping
            .clone()
            .resolve_affordance_pointer(affordance_pointer)?
            .context("Could not resolve affordance JSON Pointer against SDF model.")?;

        let protocol_mapping = ProtocolMapping::try_new(
            interaction_affordance,
            sdf_grouping.clone(),
            preferred_protocol,
        )?;

        let mut result: Option<Value> = None;

        match self {
            Operation::Read { observe } => {
                result = protocol_mapping
                    .perform_read_operation("TODO".to_string())
                    .await?;
            }
            Operation::Write { input } => todo!(),
            Operation::Invoke => todo!(),
            Operation::Subscribe => todo!(),
            Operation::Configure { input_file_name } => todo!(),
            Operation::ListConfigParameters { show_schema } => {
                Self::list_config_parameters(sdf_grouping, show_schema);
                return Ok(());
            }
        }

        if let Some(result) = result {
            io::stdout().write_all(serde_json::to_string(&result).unwrap().as_bytes())?;
        }

        Ok(())
    }

    fn list_config_parameters(target_definition: SdfGrouping, show_schema: bool) {
        let sdf_context = target_definition.sdf_context().unwrap_or_default();

        if sdf_context.is_empty() {
            eprintln!("SDF Grouping does not contain context definitions!");
            return;
        }

        let mut configurable_parameters = sdf_context
            .into_iter()
            .filter(|(_, value)| value.writable)
            .peekable();

        match configurable_parameters.peek() {
            None => {
                eprintln!("SDF Thing does not have configurable parameters!");
                return;
            }
            Some(_) => eprintln!("Configurable Parameters:"),
        }

        for (key, value) in configurable_parameters {
            eprintln!("{key}");

            if show_schema {
                eprintln!("Schema: {}", serde_json::to_string(&value).unwrap());
            }
        }
    }

    // async fn obtain_sdf_message(&self, instance_url: Url) -> anyhow::Result<SdfMessage> {
    //     match instance_url.scheme() {
    //         "http" | "https" => {
    //             let sdf_instance = reqwest::get(instance_url)
    //                 .await?
    //                 .json::<SdfMessage>()
    //                 .await?;

    //             return Ok(sdf_instance);
    //         }
    //         "coaps" => {
    //             let config = Config {
    //                 psk: Some(Arc::new(|_| Ok("secretPSK".as_bytes().to_vec()))),
    //                 cipher_suites: vec![CipherSuiteId::Tls_Psk_With_Aes_128_Ccm_8],
    //                 psk_identity_hint: Some("identity".as_bytes().to_vec()),
    //                 ..Default::default()
    //             };

    //             let port = instance_url.port().unwrap_or(5684);
    //             // TODO: Deal with cases where a non-IP is used as hostname
    //             let host = instance_url.host_str().unwrap();

    //             let dtls_config = UdpDtlsConfig {
    //                 config,
    //                 dest_addr: (host, port).to_socket_addrs().unwrap().next().unwrap(),
    //             };

    //             let client = CoAPClient::from_udp_dtls_config(dtls_config)
    //                 .await
    //                 .expect("could not create client");
    //             let domain = format!("{host}:{port}");

    //             let request = RequestBuilder::new("/.well-known/sdf/instance", Method::Get)
    //                 .domain(domain)
    //                 .build();

    //             let response = client.send(request).await.unwrap();
    //             let payload_string = String::from_utf8(response.message.payload).unwrap();

    //             let sdf_instance = serde_json::from_str(&payload_string)?;

    //             return Ok(sdf_instance);
    //         }
    //         "coap" => {
    //             let response = UdpCoAPClient::get(instance_url.as_str()).await.unwrap();
    //             let payload_string = String::from_utf8(response.message.payload).unwrap();

    //             let sdf_instance = serde_json::from_str(&payload_string)?;

    //             return Ok(sdf_instance);
    //         }
    //         _ => Err(SdfConsumerError {
    //             error_message: "Unsupported URI scheme!".to_string(),
    //         }
    //         .into()),
    //     }
    // }
}
