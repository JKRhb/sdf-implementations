// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use anyhow::bail;
use async_trait::async_trait;
use coap::UdpCoAPClient;
use reqwest::Url;
use sdf_data_structures::instance::SdfMessage;
use serde_json::Value;

use crate::consumer::ConsumedSdfProperty;
use crate::protocols::ProtocolImplementation;

trait CoapProtocolMapping {
    fn url(&self) -> String;

    fn method(&self) -> String;
}

impl CoapProtocolMapping for ConsumedSdfProperty {
    fn url(&self) -> String {
        todo!()
    }

    fn method(&self) -> String {
        todo!()
    }
}

pub(crate) struct CoapImplementation {}

#[async_trait]
impl ProtocolImplementation for CoapImplementation {
    fn supported_uri_schemes(&self) -> HashSet<&'static str> {
        HashSet::from(["coap"])
    }

    async fn perform_configuration(&self) -> anyhow::Result<()> {
        // let mut patch = HashMap::new();

        // let contents = fs::read_to_string(input_file_name)?;

        // let _path = serde_json::from_str::<serde_json::Map<String, Value>>(&contents)?;

        // patch.insert("deviceName".to_string(), json!("Reconfigured Sensor"));

        // patch.insert("location".to_string(), json!("Building 2"));

        // let sdf_message = SdfMessageBuilder::default()
        //     .info(
        //         InfoBlockBuilder::default()
        //             .message_id(Uuid::new_v4())
        //             .build()?,
        //     )
        //     .sdf_instance_of(
        //         SdfInstanceOfBuilder::default()
        //             .entry_point(
        //                 sdf_instance["sdfInstanceOf"]["entryPoint"]
        //                     .as_str()
        //                     .unwrap()
        //                     .to_string(),
        //             )
        //             .build()?,
        //     )
        //     .sdf_instance(SdfInstanceBuilder::default().sdf_context(patch).build()?)
        //     .build();

        // let payload = serde_json::to_vec(&sdf_message?)?;

        // let _response = UdpCoAPClient::post(instance_url, payload).await?;

        Ok(())
    }

    async fn perform_read_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<Value> {
        let url = consumed_sdf_property.url();
        let method = consumed_sdf_property.method();

        match method.as_str() {
            "GET" => {
                let response = UdpCoAPClient::get(&url).await?;

                let payload_string = String::from_utf8(response.message.payload)?;

                let value = serde_json::to_value(payload_string)?;

                Ok(value)
            }
            _ => bail!("hi"),
        }
    }

    async fn perform_write_operation(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
        input_value: Value,
    ) -> anyhow::Result<()> {
        let url = consumed_sdf_property.url();
        let method = consumed_sdf_property.method();

        match method.as_str() {
            "PUT" => {
                let payload = serde_json::to_vec(&input_value)?;

                UdpCoAPClient::put(&url, payload).await?;

                Ok(())
            }
            // TODO: Handle other methods as well
            _ => Ok(()),
        }
    }

    async fn perform_observe_operation(
        &self,
        _consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn obtain_sdf_snapshot(&self, _instance_url: Url) -> anyhow::Result<SdfMessage> {
        todo!()
    }
}
