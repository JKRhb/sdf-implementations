// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use reqwest::Url;
use sdf_data_structures::instance::SdfMessage;
use serde_json::{Map, Value};

use crate::protocols::common::{determine_url, obtain_method, obtain_operation};

pub(crate) struct HttpImplementation {}

impl HttpImplementation {
    pub(crate) async fn obtain_sdf_instance(self, instance_url: Url) -> anyhow::Result<SdfMessage> {
        let sdf_instance = reqwest::get(instance_url)
            .await?
            .json::<SdfMessage>()
            .await?;

        return Ok(sdf_instance);
    }

    // pub(crate) async fn handle_interaction(
    //     self,
    //     _instance_url: &String,
    //     interaction_affordance: &Map<String, Value>,
    //     sdf_model: &Value,
    //     sdf_instance: &Value,
    //     operation: &Operation,
    // ) -> anyhow::Result<Option<Value>> {
    //     let protocol_map = interaction_affordance
    //         .get("sdfProtocolMap")
    //         .context("Missing sdfProtocolMap")?;

    //     if let Some(http_protocol_map) = protocol_map.get("http").and_then(|x| x.as_object()) {
    //         match operation {
    //             Operation::Read { observe: _ } => {
    //                 return self.perform_read_operation(
    //                     http_protocol_map,
    //                     sdf_model,
    //                     sdf_instance,
    //                 )
    //                 .await;
    //             }
    //             Operation::Write { input } => {
    //                 if let Some(input) = input {
    //                     return self.perform_write_operation(
    //                         http_protocol_map,
    //                         sdf_model,
    //                         sdf_instance,
    //                         input,
    //                     )
    //                     .await;
    //                 }

    //                 bail!(SdfConsumerError {
    //                     error_message: "Missing input data for write operation".to_string()
    //                 });
    //             }
    //             _ => bail!(SdfConsumerError {
    //                 error_message: "Unsupported operation".to_string()
    //             }),
    //         }
    //     }

    //     Ok(None)
    // }

    // fn obtain_operation(self) ->

    pub(crate) async fn perform_read_operation(
        self,
        url: String,
        // http_protocol_map: &Map<String, Value>,
        // sdf_model: &Value,
        // sdf_instance: &Value,
    ) -> anyhow::Result<Option<Value>> {
        // let read_operation = obtain_operation(http_protocol_map, "read".to_string())?;

        // let url = determine_url(
        //     read_operation,
        //     http_protocol_map,
        //     sdf_instance,
        //     sdf_model,
        //     "http",
        // )?;

        // let method = obtain_method(read_operation, "GET");

        match "GET" {
            "GET" => {
                let result = reqwest::get(url).await?.json::<Value>().await?;

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    pub(crate) async fn perform_write_operation(
        self,
        http_protocol_map: &Map<String, Value>,
        sdf_model: &Value,
        sdf_instance: &Value,
        input: &Value,
    ) -> anyhow::Result<Option<Value>> {
        let write_operation = obtain_operation(http_protocol_map, "write".to_string())?;

        let url = determine_url(
            write_operation,
            http_protocol_map,
            sdf_instance,
            sdf_model,
            "http",
        )?;

        let method = obtain_method(write_operation, "PUT");

        match method.as_str() {
            "PUT" => {
                reqwest::Client::new()
                    .put(url)
                    .body(serde_json::to_string(input)?)
                    .send()
                    .await?;

                Ok(None)
            }
            // TODO: Handle other methods as well
            _ => Ok(None),
        }
    }
}
