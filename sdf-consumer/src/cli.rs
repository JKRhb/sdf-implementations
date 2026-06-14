// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{
    io::{self, IsTerminal, Write},
    sync::Arc,
};

use anyhow::bail;
use clap::{Args, Parser, Subcommand};
use reqwest::Url;
use sdf_data_structures::{instance::SdfMessage, model::SdfModel};
use serde_json::Value;

use crate::{
    consumer::SdfConsumer,
    protocols::{
        coap::{CoapImplementation, PskCallback},
        http::HttpImplementation,
    },
    util::parse_json_value,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) enum Cli {
    #[clap(flatten)]
    AffordanceOperation(AffordanceOperation),

    /// Lists all writable sdfContext definition
    ListConfigParameters(ListConfigParametersOperation),
}

#[derive(Args, Debug)]
pub(crate) struct ListConfigParametersOperation {
    /// URL pointing to a resource hosting an SDF snapshot containing the configurable parameters.
    instance_url: Url,

    /// Show the schema definition of the config parameter.
    #[clap(long, short)]
    show_schema: bool,

    #[clap(flatten)]
    security_arguments: SecurityArguments,
}

impl From<&ListConfigParametersOperation> for Option<PskCallback> {
    fn from(val: &ListConfigParametersOperation) -> Self {
        (&val.security_arguments).into()
    }
}

#[derive(Args, Debug)]
pub(crate) struct SecurityArguments {
    /// PSK that should be used when using CoAP over DTLS in PSK mode.
    #[arg(short = 'd', long)]
    pub(crate) dtls_psk: Option<String>,

    /// Identity that should be used when using CoAP over DTLS in PSK mode.
    #[arg(short = 'i', long)]
    pub(crate) dtls_identity: Option<String>,
}

impl From<&SecurityArguments> for Option<PskCallback> {
    fn from(val: &SecurityArguments) -> Self {
        if let Some(dtls_psk) = val.dtls_psk.clone() {
            let callback = Arc::new(move |_: &[u8]| Ok(Vec::from(dtls_psk.as_bytes())));

            Some(callback)
        } else {
            None
        }
    }
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
    preferred_protocol: Option<Vec<String>>,

    #[clap(flatten)]
    security_arguments: SecurityArguments,
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

        #[arg(value_parser = parse_json_value)]
        input: Value,
    },

    /// Invokes an action of an SDF Thing.
    Invoke {
        #[clap(flatten)]
        common_args: CommonAffordanceArguments,

        action_pointer: String,

        #[arg(value_parser = parse_json_value)]
        input: Option<Value>,
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

impl AffordanceOperation {
    fn common_args(&self) -> &CommonAffordanceArguments {
        match self {
            AffordanceOperation::Read {
                common_args,
                property_pointer: _,
                observe: _,
            } => common_args,
            AffordanceOperation::Write {
                common_args,
                property_pointer: _,
                input: _,
            } => common_args,
            AffordanceOperation::Invoke {
                common_args,
                action_pointer: _,
                input: _,
            } => common_args,
            AffordanceOperation::Subscribe {
                common_args,
                event_pointer: _,
            } => common_args,
            AffordanceOperation::Configure {
                common_args,
                input_file_name: _,
            } => common_args,
        }
    }

    fn security_arguments(&self) -> &SecurityArguments {
        let common_args = self.common_args();

        &common_args.security_arguments
    }
}

impl From<&AffordanceOperation> for Option<PskCallback> {
    fn from(val: &AffordanceOperation) -> Self {
        let security_arguments = val.security_arguments();

        security_arguments.into()
    }
}

impl From<&Cli> for Option<PskCallback> {
    fn from(val: &Cli) -> Self {
        match val {
            Cli::AffordanceOperation(affordance_operation) => affordance_operation.into(),
            Cli::ListConfigParameters(list_config_parameters) => list_config_parameters.into(),
        }
    }
}

pub(crate) type ObserveHandler = Box<dyn FnMut(anyhow::Result<Value>) + Send + 'static>;

impl Cli {
    fn psk_identity_hint(&self) -> Option<String> {
        match self {
            Cli::AffordanceOperation(affordance_operation) => (affordance_operation
                .security_arguments())
            .dtls_identity
            .clone(),
            Cli::ListConfigParameters(list_config_parameters_operation) => {
                list_config_parameters_operation
                    .security_arguments
                    .dtls_identity
                    .clone()
            }
        }
    }

    pub(crate) fn create_observe_handler() -> ObserveHandler {
        Box::new(|value| match value {
            Ok(value) => println!("{:?}", value),
            Err(err) => eprintln!("{err}"),
        })
    }

    pub(crate) async fn handle_operation(self) -> anyhow::Result<()> {
        let mut sdf_consumer = SdfConsumer::new();

        sdf_consumer.add_protocol_implementation(Box::from(HttpImplementation::new()))?;
        sdf_consumer.add_protocol_implementation(Box::from(CoapImplementation::new(
            (&self).into(),
            self.psk_identity_hint(),
        )))?;

        match self {
            Cli::ListConfigParameters(list_config_parameters_operation) => {
                let ListConfigParametersOperation {
                    instance_url,
                    show_schema,
                    security_arguments: _,
                } = list_config_parameters_operation;

                let (sdf_message, sdf_model) = sdf_consumer.consume_from_url(instance_url).await?;

                Self::list_config_parameters(sdf_message, sdf_model, show_schema)
            }
            Cli::AffordanceOperation(affordance_operation) => {
                let mut result: Option<Value> = None;

                match affordance_operation {
                    AffordanceOperation::Read {
                        common_args,
                        property_pointer,
                        observe,
                    } => {
                        let protocol_preference = common_args.preferred_protocol;
                        let instance_url = common_args.instance_url;

                        let (sdf_message, sdf_model) =
                            sdf_consumer.consume_from_url(instance_url).await?;

                        if observe {
                            let observe_handler = Self::create_observe_handler();

                            sdf_consumer
                                .observe_property(
                                    sdf_message,
                                    sdf_model,
                                    property_pointer,
                                    protocol_preference,
                                    observe_handler,
                                )
                                .await?;
                        } else {
                            result = Some(
                                sdf_consumer
                                    .read_property(
                                        sdf_message,
                                        sdf_model,
                                        property_pointer,
                                        protocol_preference,
                                    )
                                    .await?,
                            );
                        }
                    }
                    AffordanceOperation::Write {
                        input,
                        property_pointer,
                        common_args,
                    } => {
                        println!("{:?}", input);

                        let protocol_preference = common_args.preferred_protocol;
                        let instance_url = common_args.instance_url;

                        let (sdf_message, sdf_model) =
                            sdf_consumer.consume_from_url(instance_url).await?;

                        sdf_consumer
                            .write_property(
                                sdf_message,
                                sdf_model,
                                property_pointer,
                                protocol_preference,
                                input,
                            )
                            .await?;
                    }
                    AffordanceOperation::Invoke {
                        input,
                        action_pointer,
                        common_args,
                    } => {
                        println!("{:?}", input);

                        let protocol_preference = common_args.preferred_protocol;
                        let instance_url = common_args.instance_url;

                        let (sdf_message, sdf_model) =
                            sdf_consumer.consume_from_url(instance_url).await?;

                        sdf_consumer
                            .invoke_action(
                                sdf_message,
                                sdf_model,
                                action_pointer,
                                protocol_preference,
                                input,
                            )
                            .await?;
                    }
                    AffordanceOperation::Subscribe {
                        event_pointer: _,
                        common_args: _,
                    } => todo!(),
                    AffordanceOperation::Configure {
                        input_file_name: _,
                        common_args: _,
                    } => todo!(),
                }

                if let Some(result) = result {
                    io::stdout().write_all(serde_json::to_string(&result).unwrap().as_bytes())?;

                    if std::io::stdout().is_terminal() {
                        println!();
                    }
                }

                Ok(())
            }
        }
    }

    fn list_config_parameters(
        sdf_snapshot: SdfMessage,
        sdf_model: SdfModel,
        show_schema: bool,
    ) -> anyhow::Result<()> {
        let entry_point = sdf_snapshot.get_entry_point();

        let definitions = sdf_model.list_config_parameters(entry_point)?;

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
}
