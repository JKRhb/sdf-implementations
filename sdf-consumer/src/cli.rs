// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use crate::operation::Operation;
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
}

impl Cli {
    pub(crate) async fn handle_command(self) -> anyhow::Result<()> {
        self.operation.handle_operation().await?;

        Ok(())
    }
}
