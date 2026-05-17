// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use anyhow::bail;
use clap::{Args, Parser, Subcommand};
use reqwest::Url;
use serde_json::Value;

use crate::{
    consumer::{ConsumedSdfGrouping, SdfConsumer},
    protocols::http::HttpImplementation,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) enum Cli {
    #[clap(flatten)]
    AffordanceOperation(AffordanceOperation),

    /// Lists all writable sdfContext definition
    ListConfigParameters {
        /// URL pointing to a resource hosting an SDF snapshot containing the configurable parameters.
        instance_url: Url,

        /// Show the schema definition of the config parameter.
        #[clap(long, short)]
        show_schema: bool,
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

impl Cli {
    pub(crate) async fn handle_operation(self) -> anyhow::Result<()> {
        let mut sdf_consumer = SdfConsumer::new();

        sdf_consumer.add_protocol_implementation(Box::from(HttpImplementation::new()))?;

        match self {
            Cli::ListConfigParameters {
                show_schema,
                instance_url,
            } => {
                let consumed_sdf_grouping = sdf_consumer.consume_from_url(instance_url).await?;

                Self::list_config_parameters(consumed_sdf_grouping, show_schema)
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
                        input: _,
                        property_pointer: _,
                        common_args: _,
                    } => todo!(),
                    AffordanceOperation::Invoke {
                        action_pointer: _,
                        common_args: _,
                    } => todo!(),
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
}
