// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use anyhow::bail;
use clap::{Args, Subcommand};
use reqwest::Url;
use serde_json::Value;

use crate::{
    consumer::{ConsumedSdfGrouping, SdfConsumer},
    protocols::http::HttpImplementation,
};

#[derive(Subcommand)]
pub(crate) enum Operation {
    #[clap(flatten)]
    AffordanceOperation(AffordanceOperation),

    ListConfigParameters {
        #[clap(long, short)]
        show_schema: bool,

        /// URL pointing to a resource hosting an SDF snapshot containing the configurable parameters.
        instance_url: Url,
    },
}

#[derive(Args, Debug)]
pub(crate) struct CommonAffordanceArguments {
    /// URL pointing to a resource hosting an SDF snapshot.
    instance_url: Url,

    /// Preferred protocol map for interactions.
    ///
    /// If unset, coap will be used by default if present in the resolved
    /// model.
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    preferred_protocol: Vec<String>,
}

#[derive(Subcommand)]
pub(crate) enum AffordanceOperation {
    /// Reads a property from an SDF Thing
    Read {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        property_pointer: String,

        #[clap(long, short)]
        observe: bool,
    },

    /// Writes the property of an SDF Thing
    Write {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        property_pointer: String,

        input: Option<Value>,
    },

    /// Invokes an action of an SDF Thing.
    Invoke {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        action_pointer: String,
    },

    /// Subscribes to an event of an SDF Thing.
    Subscribe {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        event_pointer: String,
    },

    /// Reconfigures a Thing
    Configure {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        input_file_name: String,
    },
}

impl Operation {
    pub(crate) async fn handle_operation(self) -> anyhow::Result<()> {
        let mut sdf_consumer = SdfConsumer::new();

        sdf_consumer.add_protocol_implementation(Box::from(HttpImplementation::new()))?;

        match self {
            Operation::ListConfigParameters {
                show_schema,
                instance_url,
            } => {
                let consumed_sdf_grouping = sdf_consumer.consume_from_url(instance_url).await?;

                Self::list_config_parameters(consumed_sdf_grouping, show_schema)
            }
            Operation::AffordanceOperation(affordance_operation) => {
                let mut result: Option<Value> = None;

                match affordance_operation {
                    AffordanceOperation::Read {
                        common_args,
                        property_pointer,
                        observe,
                    } => {
                        let protocol_preference = common_args.preferred_protocol;
                        let instance_url = common_args.instance_url;

                        let consumed_sdf_grouping =
                            sdf_consumer.consume_from_url(instance_url).await?;

                        if observe {
                            consumed_sdf_grouping
                                .observe_property(property_pointer, protocol_preference)
                                .await?;
                        } else {
                            result = Some(
                                consumed_sdf_grouping
                                    .read_property(property_pointer, protocol_preference)
                                    .await?,
                            );
                        }
                    }
                    AffordanceOperation::Write {
                        input,
                        property_pointer,
                        common_args,
                    } => todo!(),
                    AffordanceOperation::Invoke {
                        action_pointer,
                        common_args,
                    } => todo!(),
                    AffordanceOperation::Subscribe {
                        event_pointer,
                        common_args,
                    } => todo!(),
                    AffordanceOperation::Configure {
                        input_file_name,
                        common_args,
                    } => todo!(),
                }

                if let Some(result) = result {
                    io::stdout().write_all(serde_json::to_string(&result).unwrap().as_bytes())?;
                }

                Ok(())
            }
        }
    }

    fn list_config_parameters(
        consumed_sdf_grouping: ConsumedSdfGrouping,
        show_schema: bool,
    ) -> anyhow::Result<()> {
        let definitions = consumed_sdf_grouping.list_config_parameters();

        if definitions.is_empty() {
            bail!("SDF Grouping does not contain context definitions!");
        }

        let mut configurable_parameters = definitions
            .into_iter()
            .filter(|(_, value)| value.writable)
            .peekable();

        match configurable_parameters.peek() {
            None => {
                bail!("SDF Thing does not have configurable parameters!");
            }
            Some(_) => eprintln!("Configurable Parameters:"),
        }

        for (key, value) in configurable_parameters {
            eprintln!("{key}");

            if show_schema {
                eprintln!("Schema: {}", serde_json::to_string(&value).unwrap());
            }
        }

        Ok(())
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
