use std::collections::HashMap;

use anyhow::Context;
use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::model::affordances::SdfOperation;
use crate::model::common_qualities::CommonQualities;

use crate::model::protocol_mappings::coap::EventCoapProtocolMap;
use crate::model::protocol_mappings::http::EventHttpProtocolMap;
use crate::{model::sdf_data::SdfData, traits::GlobalNameContributor};

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
pub struct SdfEvent {
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(default)]
    pub sdf_protocol_map: Option<EventProtocolMap>,

    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[builder(setter(strip_option), default)]
    pub sdf_output_data: Option<SdfData>,
    #[serde(flatten)]
    #[builder(setter(into), default)]
    pub additional_qualities: HashMap<String, Value>,
}

impl super::SdfAffordance for SdfEvent {
    fn supported_uri_schemes(&self, sdf_operation: SdfOperation) -> anyhow::Result<Vec<String>> {
        self.sdf_protocol_map
            .as_ref()
            .context("Missing SDF protocol map")?
            .supported_uri_schemes(sdf_operation)
    }
}

impl GlobalNameContributor for SdfEvent {
    const QUALITY_NAME: &'static str = "sdfEvent";
}

#[skip_serializing_none]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
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
pub struct EventProtocolMap {
    pub coap: Option<EventCoapProtocolMap>,
    pub http: Option<EventHttpProtocolMap>,
}

impl EventProtocolMap {
    pub fn supported_uri_schemes(
        &self,
        _sdf_operation: SdfOperation,
    ) -> anyhow::Result<Vec<String>> {
        todo!()
    }
}
