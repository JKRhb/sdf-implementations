use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
};

mod cli;
mod protocol_mappings;

use clap::Parser;

use ::json_pointer::JsonPointer;
use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{
    cli::Cli,
    protocol_mappings::{Operation, SupportedProtocols, common::obtain_entry_point},
};

#[derive(Debug)]
pub(crate) struct SdfConsumerError {
    pub error_message: String,
}

impl std::error::Error for SdfConsumerError {}

impl Display for SdfConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to resolved URI for prefix: {}.",
            self.error_message,
        )
    }
}

fn generate_query_string(sdf_instance: &Value) -> Option<String> {
    let mut query_parameters = HashMap::<String, String>::new();

    let sdf_instance_of = sdf_instance
        .get("sdfInstanceOf")
        .and_then(|x| x.as_object());

    for parameter_key in [
        "lineage",
        "version",
        "minVersion",
        "maxVersion",
        "exclusiveMinVersion",
        "exclusiveMaxVersion",
    ] {
        let parameter_value = sdf_instance_of
            .and_then(|x| x.get(parameter_key))
            .and_then(|x| x.as_str());

        if let Some(parameter_value) = parameter_value {
            query_parameters.insert(parameter_key.to_string(), parameter_value.to_string());
        }
    }

    if query_parameters.is_empty() {
        return None;
    }

    Some(
        query_parameters
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&"),
    )
}

fn obtain_sdf_model_url(sdf_instance: &Value) -> Result<String> {
    let default_namespace = sdf_instance
        .get("defaultNamespace")
        .and_then(|x| x.as_str())
        .context("Missing defaultNamespace")?;

    let mut model_url = sdf_instance
        .get("namespace")
        .and_then(|x| x.as_object())
        .context("Namespace is missing or not object")?
        .get(default_namespace)
        .and_then(|x| x.as_str())
        .map(|x| x.to_string())
        .ok_or(SdfConsumerError {
            // TODO: Should be part of validation process.
            error_message: "Default namespace prefix not contained in namespace map.".to_string(),
        })?;

    if let Some(query_string) = generate_query_string(sdf_instance) {
        model_url.push_str(format!("?{query_string}").as_str());
    }

    Ok(model_url)
}

fn print_config_paramters(target_definition: &Value, show_schema: bool) {
    let sdf_context = target_definition
        .get("sdfContext")
        .and_then(|x| x.as_object())
        .unwrap();

    let mut configurable_parameters = Vec::<_>::new();

    for sdf_context_entry in sdf_context {
        let is_writable = sdf_context_entry
            .1
            .as_object()
            .and_then(|x| x.get("writable"))
            .and_then(|x| x.as_bool())
            .unwrap_or(false);

        if is_writable {
            configurable_parameters.push(sdf_context_entry);
        }
    }

    if configurable_parameters.is_empty() {
        eprintln!("SDF Thing does not have configurable parameters!");
    } else {
        eprintln!("Configurable Parameters:");

        for (key, value) in configurable_parameters {
            eprintln!("{key}");

            if show_schema {
                eprintln!("Schema: {value}");
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let sdf_instance = cli.obtain_sdf_instance().await?;

    if !sdf_instance.is_object() {
        bail!(SdfConsumerError {
            error_message: "Instance URL did not return a JSON object!".to_string()
        })
    }

    let protocol_order = cli.get_protocol_preference();

    // let model_url = sdf_instance.obtain_sdf_model_url()?;
    let model_url = obtain_sdf_model_url(&sdf_instance)?;

    let sdf_model = reqwest::get(model_url).await?.json::<Value>().await?;

    println!("{:?}", sdf_model);

    let entry_point_value = obtain_entry_point(&sdf_instance)?;

    println!("{entry_point_value}");

    let entry_point_pointer = entry_point_value.parse::<JsonPointer<_, _>>().unwrap();

    let target_definition = entry_point_pointer.get(&sdf_model).unwrap();

    if let Operation::ListConfigParameters { show_schema } = cli.operation {
        print_config_paramters(target_definition, show_schema);

        return Ok(());
    }

    let affordance_pointer = cli.affordance_pointer.parse::<JsonPointer<_, _>>().unwrap();

    let interaction_affordance = affordance_pointer
        .get(&sdf_model)
        // TODO: Use correct error here
        .map_err(|_x| SdfConsumerError {
            error_message: "Failed to resolved JSON Pointer".to_string(),
        })?
        .as_object()
        .context("context")?;

    let mut result: Option<Value> = None;
    for protocol in protocol_order {
        if result.is_some() {
            break;
        }

        match protocol {
            SupportedProtocols::Coap => {
                result = protocol_mappings::coap::handle_interaction(
                    &cli.instance_url,
                    interaction_affordance,
                    &sdf_model,
                    &sdf_instance,
                    &cli.operation,
                )
                .await?;
            }
            SupportedProtocols::Http => {
                result = protocol_mappings::http::handle_interaction(
                    &cli.instance_url,
                    interaction_affordance,
                    &sdf_model,
                    &sdf_instance,
                    &cli.operation,
                )
                .await?;
            }
        }
    }

    if let Some(result) = result {
        io::stdout().write_all(serde_json::to_string(&result).unwrap().as_bytes())?;
    }

    Ok(())
}
