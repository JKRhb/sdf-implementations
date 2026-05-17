use std::{collections::HashMap, rc::Rc};

use anyhow::{Context, bail};
use reqwest::Url;
use sdf_data_structures::{
    model::{SdfContext, SdfModel, SdfProperty},
    traits::{SdfAffordance, SdfGrouping},
};

use crate::protocols::ProtocolImplementation;

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
        let supported_protocols: Vec<&String> = self.supported_protocols.keys().collect();

        for uri_scheme in &uri_schemes_to_register {
            if supported_protocols.contains(&uri_scheme) {
                bail!(format!(
                    "URI scheme {} has already been registered with this SDF consumer",
                    uri_scheme
                ))
            }
        }

        let foobar = Rc::new(protocol_implementation);

        for uri_scheme in &uri_schemes_to_register {
            self.supported_protocols
                .insert(uri_scheme.clone(), foobar.clone());
        }

        Ok(())
    }

    fn determine_protocol_implementation(
        &self,
        scheme: String,
    ) -> Option<Rc<Box<dyn ProtocolImplementation>>> {
        self.supported_protocols.get(&scheme).cloned()
    }

    pub async fn consume_from_url(self, instance_url: Url) -> anyhow::Result<ConsumedSdfGrouping> {
        let scheme = instance_url.scheme().to_string();

        let protocol_implementation = self
            .determine_protocol_implementation(scheme)
            .context("hi")?;

        let sdf_snapshot = protocol_implementation
            .as_ref()
            .as_ref()
            .obtain_sdf_snapshot(instance_url)
            .await?;

        let pointer_prefix = sdf_snapshot.get_entry_point();

        let model_url = sdf_snapshot.get_sdf_model_url()?.context("hi")?;

        let sdf_model = reqwest::get(model_url).await?.json::<SdfModel>().await?;

        // // TODO: Handle pointer prefix
        let sdf_grouping = sdf_model.resolve_entry_point_from_sdf_message(sdf_snapshot)?;

        Ok(Rc::from(self).consume(sdf_grouping, pointer_prefix))
    }

    pub(crate) fn consume(
        self: Rc<Self>,
        sdf_grouping: SdfGrouping,
        pointer_prefix: String,
    ) -> ConsumedSdfGrouping {
        ConsumedSdfGrouping {
            internal_data: sdf_grouping,
            sdf_consumer: self,
            pointer_prefix,
        }
    }

    async fn read_property(
        &self,
        consumed_sdf_property: ConsumedSdfProperty,
    ) -> anyhow::Result<serde_json::Value> {
        let scheme = "http".to_string();
        let protocol_implementation = self
            .determine_protocol_implementation(scheme)
            .context("hi")?;

        protocol_implementation
            .perform_read_operation(consumed_sdf_property)
            .await
    }
}

pub(crate) struct ConsumedSdfGrouping {
    pointer_prefix: String,
    internal_data: SdfGrouping,
    sdf_consumer: Rc<SdfConsumer>,
}

pub struct ConsumedSdfProperty {
    pub internal_data: SdfProperty,
}

impl ConsumedSdfProperty {
    pub(crate) fn supported_uri_schemes(self) -> Vec<&'static str> {
        let mut result = Vec::new();

        if let Some(sdf_protocol_map) = self.internal_data.sdf_protocol_map {
            if let Some(_coap_protocol_map) = sdf_protocol_map.coap {
                result.push("coap");
            }

            if let Some(_http_protocol_map) = sdf_protocol_map.http {
                result.push("http");
            }
        }

        result
    }
}

impl ConsumedSdfGrouping {
    pub fn list_config_parameters(self) -> HashMap<String, SdfContext> {
        self.internal_data.sdf_context().unwrap_or_default()
    }

    fn get_property(self, property_pointer: &str) -> Option<ConsumedSdfProperty> {
        let affordance = self
            .internal_data
            .resolve_affordance_pointer(property_pointer)
            .ok()??;

        match affordance {
            SdfAffordance::SdfProperty(sdf_property) => Some(ConsumedSdfProperty {
                internal_data: sdf_property,
            }),
            _ => None,
        }
    }

    pub(crate) async fn read_property(
        self,
        property_pointer: &str,
        _protocol_preference: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let sdf_consumer = self.sdf_consumer.clone();
        let consumed_sdf_property = self.get_property(property_pointer).context(format!(
            "Error obtaining sdfProperty definition via pointer {}",
            property_pointer
        ))?;

        sdf_consumer.read_property(consumed_sdf_property).await
    }

    pub(crate) async fn observe_property(
        self,
        property_pointer: &str,
        _protocol_preference: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let sdf_consumer = self.sdf_consumer.clone();
        let consumed_sdf_property = self.get_property(property_pointer).context(format!(
            "Error obtaining sdfProperty definition via pointer {}",
            property_pointer
        ))?;

        sdf_consumer.read_property(consumed_sdf_property).await
    }

    pub(crate) async fn write_property(
        self,
        property_pointer: &str,
        _protocol_preference: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let sdf_consumer = self.sdf_consumer.clone();
        let consumed_sdf_property = self.get_property(property_pointer).context(format!(
            "Error obtaining sdfProperty definition via pointer {}",
            property_pointer
        ))?;

        sdf_consumer.read_property(consumed_sdf_property).await
    }
}
