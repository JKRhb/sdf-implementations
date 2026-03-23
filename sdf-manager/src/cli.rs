use clap::{Parser, Subcommand};

/// The subcommands exposed by the SDF Manager.
#[derive(Subcommand)]
pub(crate) enum Operation {
    /// Registers a new model that is read from the given file path.
    Register {
        /// Location of the SDF model input file.
        #[clap(short, long)]
        input_file: String,
    },

    /// Updates a model using a supplement that is read from the given file path.
    Update {
        /// Location of the SDF supplement input file.
        #[clap(short, long)]
        input_file: String,
    },

    /// Deletes all models from a given lineage.
    ///
    /// Optionally allows for selecting an (inclusive) minimal version for deletion.
    Delete {
        /// Namespace URL of the models that should be deleted.
        target_namespace: String,

        /// Lineage identifier of the models that should be deleted.
        #[clap(short, long)]
        lineage: Option<String>,

        /// Minimal version of the models that should be deleted.
        #[clap(short, long)]
        min_version: Option<String>,
    },

    /// Lists all models under the given namespace according to the provided filters.
    List {
        /// Namespace URL of the models that should be queried
        target_namespace: String,

        /// The requested lineage.
        #[clap(short, long)]
        lineage: Option<String>,

        /// The exact version of a target model.
        #[clap(short, long)]
        version: Option<String>,

        /// A minimal model version.
        #[clap(short = 'n', long)]
        min_version: Option<String>,

        /// A maximal model version.
        #[clap(short, long)]
        max_version: Option<String>,

        /// A (exclusively) minimal version.
        #[clap(short = 'e', long)]
        exclusive_min_version: Option<String>,

        /// A (exclusively) maximal version.
        #[clap(short = 'x', long)]
        exclusive_max_version: Option<String>,
    },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// The operation that is to performed with the SDF Repository.
    #[command(subcommand)]
    pub(crate) operation: Operation,
}

impl Cli {}
