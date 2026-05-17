// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CoapProtocolMap {
    method: Option<String>,
    href: String,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PropertyCoapOperations {
    #[serde(flatten)]
    protocol_map: CoapProtocolMap,
    content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActionCoapProtocolMap {
    #[serde(flatten)]
    protocol_map: CoapProtocolMap,
    input_content_format: Option<Vec<u32>>,
    output_content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct EventCoapProtocolMap {
    #[serde(flatten)]
    protocol_map: CoapProtocolMap,
    output_content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CoapProtocolMapParameters {
    host_names: Option<String>,
    ip_address: Option<String>,
    // TODO: Additional parameters?
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CoapPropertyOperations {
    read: Option<PropertyCoapOperations>,
    write: Option<PropertyCoapOperations>,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PropertyCoapProtocolMap {
    sdf_parameters: Option<CoapProtocolMapParameters>,
    sdf_operations: Option<CoapPropertyOperations>,
}
