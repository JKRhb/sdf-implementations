use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpProtocolMap {
    pub method: String,
    pub href: String,
    pub headers: Option<HashMap<String, String>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpProtocolMapParameters {
    pub host: Option<String>,
    pub ip_address: Option<String>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpPropertyOperations {
    pub read: Option<HttpProtocolMap>,
    pub write: Option<HttpProtocolMap>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertyHttpProtocolMap {
    pub sdf_parameters: Option<HttpProtocolMapParameters>,

    pub sdf_operations: Option<HttpPropertyOperations>,
}
