use ::coap::UdpCoAPClient;
use clap::{Subcommand, ValueEnum};
use serde_json::{Map, Value};

use crate::protocol_mappings::common::{determine_url, obtain_operation};

pub(crate) mod coap;
pub(super) mod common;
pub(crate) mod http;

#[derive(Clone, Copy, ValueEnum, PartialEq, Debug)]
pub(crate) enum SupportedProtocols {
    Coap,
    Http,
}

#[derive(Subcommand)]
pub(crate) enum Operation {
    /// Reads a property from an SDF Thing
    Read {
        #[clap(long, short)]
        observe: bool,
    },

    /// Writes the property of an SDF Thing
    Write { input: Option<Value> },

    /// Invokes an action of an SDF Thing.
    Invoke,

    /// Subscribes to an event of an SDF Thing.
    Subscribe,

    /// Reconfigures a Thing
    Configure { input_file_name: String },

    ListConfigParameters {
        #[clap(long, short)]
        show_schema: bool,
    },
}

// TODO: Maybe needs better name
trait ProtocolMapping {
    fn supported_uri_schemes() -> Vec<String>;

    async fn perform_read_operation(
        protocol_map: &Map<String, Value>,
        sdf_model: &Value,
        sdf_instance: &Value,
    ) -> anyhow::Result<Option<Value>>;
}

struct CoapProtocolMapping {}

impl ProtocolMapping for CoapProtocolMapping {
    fn supported_uri_schemes() -> Vec<String> {
        todo!()
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

        let method = common::obtain_method(read_operation, "GET");

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
}
