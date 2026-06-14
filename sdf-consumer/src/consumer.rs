// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, rc::Rc};

use anyhow::{Context, Ok, bail};
use reqwest::Url;
use sdf_data_structures::{
    instance::SdfMessage,
    model::{
        SdfModel,
        affordances::{
            SdfAffordance, SdfOperation, sdf_action::SdfAction, sdf_property::SdfProperty,
        },
    },
};
use serde_json::Value;

use crate::{cli::ObserveHandler, protocols::ProtocolImplementation};

pub struct SdfConsumer {
    supported_protocols: HashMap<String, Rc<Box<dyn ProtocolImplementation>>>,
}

impl SdfConsumer {
    pub fn new() -> Self {
        Self {
            supported_protocols: HashMap::new(),
        }
    }

    pub fn add_protocol_implementation(
        &mut self,
        protocol_implementation: Box<dyn ProtocolImplementation>,
    ) -> anyhow::Result<()> {
        let uri_schemes_to_register = protocol_implementation.supported_uri_schemes().clone();
        let supported_protocols: Vec<&str> = self
            .supported_protocols
            .keys()
            .map(|x| x.as_str())
            .collect();

        for uri_scheme in &uri_schemes_to_register {
            if supported_protocols.contains(uri_scheme) {
                bail!(format!(
                    "URI scheme {} has already been registered with this SDF consumer",
                    uri_scheme
                ))
            }
        }

        let protocol_implementation = Rc::new(protocol_implementation);

        for uri_scheme in uri_schemes_to_register {
            self.supported_protocols
                .insert(uri_scheme.to_string(), protocol_implementation.clone());
        }

        Ok(())
    }

    fn determine_protocol_implementation(
        &self,
        uri_scheme: &str,
    ) -> anyhow::Result<Rc<Box<dyn ProtocolImplementation>>> {
        self.supported_protocols
            .get(uri_scheme)
            .cloned()
            .context("hi")
    }

    fn determine_scheme_and_protocol_implementation(
        &self,
        sdf_affordance: Box<dyn SdfAffordance>,
        sdf_operation: SdfOperation,
        protocol_preference: Option<Vec<String>>,
    ) -> anyhow::Result<(String, Rc<Box<dyn ProtocolImplementation>>)> {
        let uri_scheme =
            self.determine_uri_scheme(sdf_affordance, sdf_operation, protocol_preference)?;

        let protocol_implementation = self.determine_protocol_implementation(&uri_scheme)?;

        Ok((uri_scheme, protocol_implementation))
    }

    pub async fn consume_from_url(
        &self,
        instance_url: Url,
    ) -> anyhow::Result<(SdfMessage, SdfModel)> {
        let scheme = instance_url.scheme();

        let protocol_implementation = self.determine_protocol_implementation(scheme)?;

        let sdf_snapshot = protocol_implementation
            .as_ref()
            .as_ref()
            .obtain_sdf_snapshot(instance_url)
            .await?;

        let model_url = sdf_snapshot
            .get_sdf_model_url()?
            .context("Failed to obtain the URL of the SDF model for the retrieved SDF message.")?;

        let sdf_model = reqwest::get(model_url).await?.json::<SdfModel>().await?;

        Ok((sdf_snapshot, sdf_model))
    }
    fn determine_uri_scheme(
        &self,
        sdf_affordance: Box<dyn SdfAffordance>,
        sdf_operation: SdfOperation,
        protocol_preference: Option<Vec<String>>,
    ) -> anyhow::Result<String> {
        let supported_uri_schemes = sdf_affordance
            .as_ref()
            .supported_uri_schemes(sdf_operation)?;

        let scheme;

        if let Some(protocol_preference) = protocol_preference {
            scheme = protocol_preference
                .into_iter().find(|x| supported_uri_schemes.contains(x))
                .context("None of the preferred URI schemes are compatible with the URI schemes supported by the SDF Thing.")?;
        } else {
            scheme = supported_uri_schemes.first()
                .context("No available URI schemes are compatible with the URI schemes supported by the SDF Thing.")?
                .to_string();
        }

        Ok(scheme)
    }

    pub(crate) async fn read_property(
        &self,
        sdf_message: SdfMessage,
        sdf_model: SdfModel,
        property_pointer: String,
        protocol_preference: Option<Vec<String>>,
    ) -> anyhow::Result<serde_json::Value> {
        let sdf_property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, &sdf_model)?
            .clone();

        let (scheme, protocol_implementation) = self.determine_scheme_and_protocol_implementation(
            Box::new(sdf_property),
            SdfOperation::Read,
            protocol_preference,
        )?;

        protocol_implementation
            .perform_read_operation(
                &scheme,
                sdf_message,
                &sdf_model,
                property_pointer.to_string(),
            )
            .await
    }

    pub(crate) async fn observe_property(
        &self,
        sdf_message: SdfMessage,
        sdf_model: SdfModel,
        property_pointer: String,
        protocol_preference: Option<Vec<String>>,
        observe_handler: ObserveHandler,
    ) -> anyhow::Result<()> {
        let sdf_property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, &sdf_model)?
            .clone();

        let (scheme, protocol_implementation) = self.determine_scheme_and_protocol_implementation(
            Box::new(sdf_property),
            SdfOperation::Observe,
            protocol_preference,
        )?;

        protocol_implementation
            .perform_observe_operation(
                &scheme,
                sdf_message,
                &sdf_model,
                property_pointer.to_string(),
                observe_handler,
            )
            .await
    }

    pub(crate) async fn write_property(
        &self,
        sdf_message: SdfMessage,
        sdf_model: SdfModel,
        property_pointer: String,
        protocol_preference: Option<Vec<String>>,
        input_value: serde_json::Value,
    ) -> anyhow::Result<()> {
        let sdf_property = sdf_message
            .resolve_pointer_against_model::<&SdfProperty>(&property_pointer, &sdf_model)?
            .clone();

        let (scheme, protocol_implementation) = self.determine_scheme_and_protocol_implementation(
            Box::new(sdf_property),
            SdfOperation::Write,
            protocol_preference,
        )?;

        protocol_implementation
            .perform_write_operation(
                &scheme,
                sdf_message,
                &sdf_model,
                property_pointer.to_string(),
                input_value,
            )
            .await
    }

    pub(crate) async fn invoke_action(
        &self,
        sdf_message: SdfMessage,
        sdf_model: SdfModel,
        action_pointer: String,
        protocol_preference: Option<Vec<String>>,
        input_value: Option<serde_json::Value>,
    ) -> anyhow::Result<Option<Value>> {
        let sdf_action = sdf_message
            .resolve_pointer_against_model::<&SdfAction>(&action_pointer, &sdf_model)?
            .clone();

        let (scheme, protocol_implementation) = self.determine_scheme_and_protocol_implementation(
            Box::new(sdf_action),
            SdfOperation::Invoke,
            protocol_preference,
        )?;

        protocol_implementation
            .perform_invoke_operation(
                &scheme,
                sdf_message,
                &sdf_model,
                action_pointer.to_string(),
                input_value,
            )
            .await
    }
}
