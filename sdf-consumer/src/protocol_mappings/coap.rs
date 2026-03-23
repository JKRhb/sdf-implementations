use std::{collections::HashMap, fs};

use anyhow::{Context, bail};
use coap::UdpCoAPClient;
use sdf_data_structures::instance::{
    InfoBlockBuilder, SdfInstanceBuilder, SdfInstanceOfBuilder, SdfMessageBuilder,
};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    SdfConsumerError,
    protocol_mappings::{
        Operation,
        common::{determine_url, obtain_method, obtain_operation},
    },
};

pub async fn handle_interaction(
    instance_url: &String,
    interaction_affordance: &Map<String, Value>,
    sdf_model: &Value,
    sdf_instance: &Value,
    operation: &Operation,
) -> anyhow::Result<Option<Value>> {
    let protocol_map = interaction_affordance
        .get("sdfProtocolMap")
        .context("Missing sdfProtocolMap")?;

    if let Some(protocol_map) = protocol_map.get("coap").and_then(|x| x.as_object()) {
        match operation {
            Operation::Read { observe: _ } => {
                return perform_read_operation(protocol_map, sdf_model, sdf_instance).await;
            }
            Operation::Write { input } => {
                if let Some(input) = input {
                    return perform_write_operation(protocol_map, sdf_model, sdf_instance, input)
                        .await;
                }

                bail!(SdfConsumerError {
                    error_message: "Missing input data for write operation".to_string()
                });
            }
            Operation::Configure { input_file_name } => {
                perform_configuration(
                    instance_url,
                    input_file_name,
                    protocol_map,
                    sdf_model,
                    sdf_instance,
                )
                .await?;
            }
            _ => bail!(SdfConsumerError {
                error_message: "Unsupported operation".to_string()
            }),
        }
    }

    Ok(None)
}

async fn perform_configuration(
    instance_url: &String,
    input_file_name: &String,
    _protocol_map: &Map<String, Value>,
    _sdf_model: &Value,
    sdf_instance: &Value,
) -> anyhow::Result<()> {
    let mut patch = HashMap::new();

    let contents = fs::read_to_string(input_file_name)?;

    let path = serde_json::from_str::<serde_json::Map<String, Value>>(&contents)?;

    patch.insert("deviceName".to_string(), json!("Reconfigured Sensor"));

    patch.insert("location".to_string(), json!("Building 2"));

    let sdf_message = SdfMessageBuilder::default()
        .info(
            InfoBlockBuilder::default()
                .message_id(Uuid::new_v4())
                .build()?,
        )
        .sdf_instance_of(
            SdfInstanceOfBuilder::default()
                .entry_point(
                    sdf_instance["sdfInstanceOf"]["entryPoint"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                )
                .build()?,
        )
        .sdf_instance(SdfInstanceBuilder::default().sdf_context(patch).build()?)
        .build();

    let payload = serde_json::to_vec(&sdf_message?)?;

    let response = UdpCoAPClient::post(instance_url, payload).await?;

    Ok(())
}

async fn perform_read_operation(
    protocol_map: &Map<String, Value>,
    sdf_model: &Value,
    sdf_instance: &Value,
) -> anyhow::Result<Option<Value>> {
    let read_operation = obtain_operation(protocol_map, "read".to_string())?;

    let url = determine_url(
        read_operation,
        protocol_map,
        sdf_instance,
        sdf_model,
        "coap",
    )?;

    let method = obtain_method(read_operation, "GET");

    match method.as_str() {
        "GET" => {
            let response = UdpCoAPClient::get(&url).await?;

            let payload_string = String::from_utf8(response.message.payload)?;

            let value = serde_json::to_value(payload_string)?;

            Ok(Some(value))
        }
        _ => Ok(None),
    }
}

async fn perform_write_operation(
    coap_protocol_map: &Map<String, Value>,
    sdf_model: &Value,
    sdf_instance: &Value,
    input: &Value,
) -> anyhow::Result<Option<Value>> {
    let write_operation = obtain_operation(coap_protocol_map, "write".to_string())?;

    let url = determine_url(
        write_operation,
        coap_protocol_map,
        sdf_instance,
        sdf_model,
        "coap",
    )?;

    let method = obtain_method(write_operation, "PUT");

    match method.as_str() {
        "PUT" => {
            let payload = serde_json::to_vec(input)?;

            UdpCoAPClient::put(&url, payload).await?;

            Ok(None)
        }
        // TODO: Handle other methods as well
        _ => Ok(None),
    }
}
