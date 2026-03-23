use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoapProtocolMap {
    method: String,
    href: String,
}

pub struct PropertyCoapProtocolMap {
    protocol_map: CoapProtocolMap,
    content_format: Vec<Option<u32>>,
}

pub struct ActionCoapProtocolMap {
    protocol_map: CoapProtocolMap,
    input_content_format: Vec<Option<u32>>,
    output_content_format: Vec<Option<u32>>,
}

pub struct EventCoapProtocolMap {
    protocol_map: CoapProtocolMap,
    output_content_format: Vec<Option<u32>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoapProtocolMapParameters {
    host_names: Option<String>,
    ip_address: Option<String>,
    // TODO: Additional parameters?
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoapPropertyOperations {
    read: Option<CoapProtocolMap>,
    write: Option<CoapProtocolMap>,
}

enum CoapAffordances {}

// #[skip_serializing_none]
// #[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct PropertyCoapProtocolMap {
//     sdf_parameters: Option<CoapProtocolMapParameters>,

//     sdf_operations: Option<CoapPropertyOperations>,
// }
