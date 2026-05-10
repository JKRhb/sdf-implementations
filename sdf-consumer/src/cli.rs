// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use crate::{operation::Operation, protocols::SupportedProtocols};
use clap::Parser;
use reqwest::Url;
use thiserror::Error;

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
    pub(crate) instance_url: Url,

    /// Preferred protocol map for interactions.
    ///
    /// If unset, coap will be used by default if present in the resolved
    /// model.
    preferred_protocol: Option<SupportedProtocols>,
}

impl Cli {
    pub(crate) async fn handle_command(self) -> anyhow::Result<()> {
        self.operation
            .handle_operation(
                self.instance_url,
                self.preferred_protocol,
            )
            .await?;

        // let sdf_message = self.obtain_sdf_message().await?;

        // let protocol_order = self.get_protocol_preference();

        // let model_url = sdf_message
        //     .get_sdf_model_url()?
        //     .context("Missing SDF Model URL in SDF Message")?;

        // let sdf_model = reqwest::get(model_url).await?.json::<SdfModel>().await?;

        // let entry_point_pointer = sdf_message.get_entry_point();

        // let entry_point_definition = sdf_model.resolve_entry_point_from_sdf_message(sdf_message)?;

        // if let Operation::ListConfigParameters { show_schema } = self.operation {
        //     print_config_parameters(entry_point_definition, show_schema);

        //     return Ok(());
        // }

        // let affordance_pointer = self
        //     .affordance_pointer
        //     .parse::<JsonPointer<_, _>>()
        //     .unwrap();

        // let interaction_affordance = affordance_pointer
        //     .get(&sdf_model)
        //     // TODO: Use correct error here
        //     .map_err(|_x| SdfConsumerError {
        //         error_message: "Failed to resolved JSON Pointer".to_string(),
        //     })?
        //     .as_object()
        //     .context("context")?;

        // let mut result: Option<Value> = None;
        // for protocol in protocol_order {
        //     if result.is_some() {
        //         break;
        //     }

        //     match protocol {
        //         SupportedProtocols::Coap => {
        //             result = protocol_mappings::coap::handle_interaction(
        //                 &cli.instance_url,
        //                 interaction_affordance,
        //                 &sdf_model,
        //                 &sdf_message,
        //                 &cli.operation,
        //             )
        //             .await?;
        //         }
        //         SupportedProtocols::Http => {
        //             result = protocol_mappings::http::handle_interaction(
        //                 &cli.instance_url,
        //                 interaction_affordance,
        //                 &sdf_model,
        //                 &sdf_message,
        //                 &cli.operation,
        //             )
        //             .await?;
        //         }
        //     }
        // }

        Ok(())
    }
}
