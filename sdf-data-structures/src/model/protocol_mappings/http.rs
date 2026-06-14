// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use anyhow::{Context, bail};
use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    instance::SdfMessage,
    model::{
        SdfEvent, SdfModel,
        affordances::{sdf_action::SdfAction, sdf_property::SdfProperty},
        sdf_context::SdfContext,
    },
};

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct HttpProtocolMap {
    pub method: String,
    pub href: String,
    pub headers: Option<HashMap<String, String>>,

    #[serde(default = "default_http_uri_scheme")]
    pub uri_schemes: Vec<String>,
}

fn default_http_uri_scheme() -> Vec<String> {
    vec!["http".to_string()]
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PropertyHttpOperations {
    #[serde(flatten)]
    pub protocol_map: HttpProtocolMap,
    // pub content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActionHttpOperations {
    #[serde(flatten)]
    pub protocol_map: HttpProtocolMap,
    // pub input_content_format: Option<Vec<u32>>,
    // pub output_content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct EventHttpOperations {
    #[serde(flatten)]
    pub protocol_map: HttpProtocolMap,
    // pub output_content_format: Option<Vec<u32>>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct HttpProtocolMapParameters {
    pub host: Option<String>,
    pub ip_address: Option<String>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct HttpPropertyOperations {
    pub read: Option<PropertyHttpOperations>,
    pub write: Option<PropertyHttpOperations>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct HttpActionOperations {
    pub invoke: Option<ActionHttpOperations>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct HttpEventOperations {
    pub subscribe: Option<EventHttpOperations>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PropertyHttpProtocolMap {
    pub sdf_parameters: Option<HttpProtocolMapParameters>,

    pub sdf_operations: Option<HttpPropertyOperations>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActionHttpProtocolMap {
    pub sdf_parameters: Option<HttpProtocolMapParameters>,

    pub sdf_operations: Option<HttpActionOperations>,
}

#[skip_serializing_none]
#[derive(
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Builder,
    Clone,
    JsonPointee,
    JsonPointerTarget,
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct EventHttpProtocolMap {
    pub sdf_parameters: Option<HttpProtocolMapParameters>,

    pub sdf_operations: Option<HttpEventOperations>,
}

pub trait HttpAffordance {
    fn ip_address(&self, sdf_model: &SdfModel, sdf_message: &SdfMessage) -> anyhow::Result<String> {
        let ip_address_pointer = self
            .ip_address_pointer()
            .context("Missing IP address context definition")?;

        let _context_definition = sdf_message
            .resolve_absolute_pointer_against_model::<&SdfContext>(&ip_address_pointer, sdf_model)?
            .to_owned();

        // TODO: Validate result
        let ip_address =
            sdf_message.resolve_pointer_against_instance::<&Value>(&ip_address_pointer)?;

        if let Value::String(ip_address) = ip_address {
            Ok(ip_address.clone())
        } else {
            bail!("Invalid type for IP address.")
        }
    }

    fn host_name(&self, sdf_model: &SdfModel, sdf_message: &SdfMessage) -> anyhow::Result<String> {
        let host_name_pointer = self
            .host_name_pointer()
            .context("Missing IP address context definition")?;

        let _context_definition = sdf_message
            .resolve_absolute_pointer_against_model::<&SdfContext>(&host_name_pointer, sdf_model)?
            .to_owned();

        // TODO: Validate result
        let ip_address =
            sdf_message.resolve_pointer_against_instance::<&Value>(&host_name_pointer)?;

        if let Value::String(ip_address) = ip_address {
            Ok(ip_address.clone())
        } else {
            bail!("Invalid type for IP address.")
        }
    }

    fn ip_address_pointer(&self) -> Option<String>;

    fn host_name_pointer(&self) -> Option<String>;

    fn host_component(
        &self,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let hostname = self.host_name(sdf_model, sdf_message);

        self.ip_address(sdf_model, sdf_message).or(hostname)
    }
}

// TODO: Refactor
impl SdfProperty {
    fn http_sdf_operations(&self) -> Option<HttpPropertyOperations> {
        self.sdf_protocol_map.clone()?.http?.sdf_operations
    }

    fn http_sdf_parameters(&self) -> Option<HttpProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.http?.sdf_parameters
    }

    fn http_read_protocol_map(&self) -> Option<HttpProtocolMap> {
        self.http_sdf_operations()?.read.map(|x| x.protocol_map)
    }

    fn http_write_protocol_map(&self) -> Option<HttpProtocolMap> {
        self.http_sdf_operations()?.write.map(|x| x.protocol_map)
    }
}

impl SdfAction {
    fn http_sdf_operations(&self) -> Option<HttpActionOperations> {
        self.sdf_protocol_map.clone()?.http?.sdf_operations
    }

    fn http_sdf_parameters(&self) -> Option<HttpProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.http?.sdf_parameters
    }

    fn http_invoke_protocol_map(&self) -> Option<HttpProtocolMap> {
        self.http_sdf_operations()?.invoke.map(|x| x.protocol_map)
    }
}

impl SdfEvent {
    #[allow(dead_code)]
    fn http_sdf_operations(&self) -> Option<HttpEventOperations> {
        self.sdf_protocol_map.clone()?.http?.sdf_operations
    }

    #[allow(dead_code)]
    fn http_sdf_parameters(&self) -> Option<HttpProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.http?.sdf_parameters
    }

    #[allow(dead_code)]
    fn subscribe_protocol_map(&self) -> Option<HttpProtocolMap> {
        self.http_sdf_operations()?
            .subscribe
            .map(|x| x.protocol_map)
    }
}

pub trait HttpProperty: HttpAffordance {
    fn read_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String>;

    fn read_method(&self) -> String;

    fn write_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String>;

    fn write_method(&self) -> String;
}

impl HttpProperty for SdfProperty {
    fn read_method(&self) -> String {
        self.http_read_protocol_map()
            .map(|x| x.method)
            .unwrap_or("GET".to_string())
            .into()
    }

    fn write_method(&self) -> String {
        self.http_write_protocol_map()
            .map(|x| x.method)
            .unwrap_or("PUT".to_string())
            .into()
    }

    fn write_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .http_write_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }

    fn read_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .http_read_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }
}

impl HttpAffordance for SdfProperty {
    fn ip_address_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.host
    }
}

pub trait HttpEvent: HttpAffordance {
    fn suscribe_method(&self, sdf_model: &SdfModel, sdf_message: &SdfMessage) -> String;
}

impl HttpAffordance for SdfEvent {
    fn ip_address_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.host
    }
}

impl HttpEvent for SdfEvent {
    fn suscribe_method(&self, _sdf_model: &SdfModel, _sdf_message: &SdfMessage) -> String {
        todo!("There is no default subscribe method defined for HTTP, yet!")
    }
}

pub trait HttpAction: HttpAffordance {
    fn invoke_method(&self, sdf_model: &SdfModel, sdf_message: &SdfMessage) -> String;
}

impl HttpAffordance for SdfAction {
    fn ip_address_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.http_sdf_parameters()?.host
    }
}

impl HttpAction for SdfAction {
    fn invoke_method(&self, _sdf_model: &SdfModel, _sdf_message: &SdfMessage) -> String {
        self.http_invoke_protocol_map()
            .map(|x| x.method)
            .unwrap_or("POST".to_string())
            .into()
    }
}
