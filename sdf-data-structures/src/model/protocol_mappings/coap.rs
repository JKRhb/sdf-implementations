// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use anyhow::{Context, bail};
use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointeeExt, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    instance::SdfMessage,
    model::{
        SdfContext, SdfEvent, SdfModel,
        affordances::{sdf_action::SdfAction, sdf_property::SdfProperty},
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
pub struct CoapProtocolMap {
    pub method: Option<CoapMethod>,
    pub href: String,

    #[serde(default = "default_coap_uri_scheme")]
    pub uri_schemes: Vec<String>,
}

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonPointee, JsonPointerTarget)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "UPPERCASE")]
pub enum CoapMethod {
    Get,
    Delete,
    Post,
    Put,
    Patch,
    #[serde(rename = "iPATCH")]
    Ipatch,
    Fetch,
}

impl From<CoapMethod> for String {
    fn from(val: CoapMethod) -> Self {
        let result = match val {
            CoapMethod::Get => "GET",
            CoapMethod::Delete => "DELETE",
            CoapMethod::Post => "POST",
            CoapMethod::Put => "PUT",
            CoapMethod::Patch => "PATCH",
            CoapMethod::Ipatch => "iPATCH",
            CoapMethod::Fetch => "FETCH",
        };

        result.to_string()
    }
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
pub struct PropertyCoapOperations {
    #[serde(flatten)]
    pub protocol_map: CoapProtocolMap,
    pub content_format: Option<Vec<u32>>,
}

fn default_coap_uri_scheme() -> Vec<String> {
    vec!["coap".to_string()]
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
pub struct ActionCoapOperations {
    #[serde(flatten)]
    pub protocol_map: CoapProtocolMap,
    pub input_content_format: Option<Vec<u32>>,
    pub output_content_format: Option<Vec<u32>>,
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
pub struct EventCoapOperations {
    #[serde(flatten)]
    pub protocol_map: CoapProtocolMap,
    pub output_content_format: Option<Vec<u32>>,
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
pub struct ActionCoapProtocolMap {
    pub sdf_parameters: Option<CoapProtocolMapParameters>,
    pub sdf_operations: Option<CoapActionOperations>,
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
pub struct EventCoapProtocolMap {
    pub sdf_parameters: Option<CoapProtocolMapParameters>,
    pub sdf_operations: Option<CoapEventOperations>,
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
pub struct CoapProtocolMapParameters {
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
}

impl CoapProtocolMapParameters {
    pub fn resolve_hostname(self, sdf_model: &SdfModel) -> anyhow::Result<String> {
        let pointer = self.hostname.context("Missing hostname quality")?;

        let hostname: &str = sdf_model.pointer(&pointer)?;

        Ok(hostname.to_string())
    }

    pub fn resolve_ip_address(self, sdf_model: &SdfModel) -> anyhow::Result<String> {
        let pointer = self.ip_address.context("Missing hostname quality")?;

        let hostname: &str = sdf_model.pointer(&pointer)?;

        Ok(hostname.to_string())
    }
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
pub struct CoapPropertyOperations {
    pub read: Option<PropertyCoapOperations>,
    pub write: Option<PropertyCoapOperations>,
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
pub struct CoapActionOperations {
    pub invoke: Option<ActionCoapOperations>,
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
pub struct CoapEventOperations {
    pub subscribe: Option<EventCoapOperations>,
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
pub struct PropertyCoapProtocolMap {
    pub sdf_parameters: Option<CoapProtocolMapParameters>,
    pub sdf_operations: Option<CoapPropertyOperations>,
}

pub trait CoapAffordance {
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

pub trait CoapProperty: CoapAffordance {
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

// TODO: Refactor
impl SdfProperty {
    fn coap_sdf_operations(&self) -> Option<CoapPropertyOperations> {
        self.sdf_protocol_map.clone()?.coap?.sdf_operations
    }

    fn coap_sdf_parameters(&self) -> Option<CoapProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.coap?.sdf_parameters
    }

    fn coap_read_protocol_map(&self) -> Option<CoapProtocolMap> {
        self.coap_sdf_operations()?.read.map(|x| x.protocol_map)
    }

    fn coap_write_protocol_map(&self) -> Option<CoapProtocolMap> {
        self.coap_sdf_operations()?.write.map(|x| x.protocol_map)
    }
}

impl SdfAction {
    fn coap_sdf_operations(&self) -> Option<CoapActionOperations> {
        self.sdf_protocol_map.clone()?.coap?.sdf_operations
    }

    fn coap_sdf_parameters(&self) -> Option<CoapProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.coap?.sdf_parameters
    }

    fn coap_invoke_protocol_map(&self) -> Option<CoapProtocolMap> {
        self.coap_sdf_operations()?.invoke.map(|x| x.protocol_map)
    }
}

impl SdfEvent {
    fn coap_sdf_operations(&self) -> Option<CoapEventOperations> {
        self.sdf_protocol_map.clone()?.coap?.sdf_operations
    }

    fn coap_sdf_parameters(&self) -> Option<CoapProtocolMapParameters> {
        self.sdf_protocol_map.clone()?.coap?.sdf_parameters
    }

    fn coap_subscribe_protocol_map(&self) -> Option<CoapProtocolMap> {
        self.coap_sdf_operations()?
            .subscribe
            .map(|x| x.protocol_map)
    }
}

impl CoapProperty for SdfProperty {
    fn read_method(&self) -> String {
        self.coap_read_protocol_map()
            .and_then(|x| x.method)
            .unwrap_or(CoapMethod::Get)
            .into()
    }

    fn write_method(&self) -> String {
        self.coap_write_protocol_map()
            .and_then(|x| x.method)
            .unwrap_or(CoapMethod::Put)
            .into()
    }

    fn read_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .coap_read_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }

    fn write_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .coap_write_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }
}

impl CoapAffordance for SdfProperty {
    fn ip_address_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.hostname
    }
}

pub trait CoapEvent: CoapAffordance {
    fn subscribe_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String>;

    fn suscribe_method(&self) -> String;
}

impl CoapAffordance for SdfEvent {
    fn ip_address_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.hostname
    }
}

impl CoapEvent for SdfEvent {
    fn suscribe_method(&self) -> String {
        self.coap_subscribe_protocol_map()
            .and_then(|x| x.method)
            .unwrap_or(CoapMethod::Get)
            .into()
    }

    fn subscribe_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .coap_subscribe_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }
}

pub trait CoapAction: CoapAffordance {
    fn invoke_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String>;

    fn invoke_method(self) -> String;
}

impl CoapAffordance for SdfAction {
    fn ip_address_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.ip_address
    }

    fn host_name_pointer(&self) -> Option<String> {
        self.coap_sdf_parameters()?.hostname
    }
}

impl CoapAction for SdfAction {
    fn invoke_method(self) -> String {
        self.coap_invoke_protocol_map()
            .and_then(|x| x.method)
            .unwrap_or(CoapMethod::Post)
            .into()
    }

    fn invoke_url(
        &self,
        scheme: &str,
        sdf_model: &SdfModel,
        sdf_message: &SdfMessage,
    ) -> anyhow::Result<String> {
        let host_component = self.host_component(sdf_model, sdf_message)?;

        let href = self
            .coap_invoke_protocol_map()
            .map(|x| x.href)
            .context("Missing href")?;

        let url = scheme.to_string() + "://" + &host_component + &href;

        Ok(url)
    }
}
