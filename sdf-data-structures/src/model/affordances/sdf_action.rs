use std::collections::HashMap;

use anyhow::Context;
use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    model::{
        affordances::SdfOperation,
        common_qualities::CommonQualities,
        protocol_mappings::{coap::ActionCoapProtocolMap, http::ActionHttpProtocolMap},
        sdf_data::SdfData,
    },
    traits::GlobalNameContributor,
    util::none_extra,
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
pub struct SdfAction {
    #[serde(flatten)]
    #[builder(default)]
    pub common_qualities: CommonQualities,

    #[builder(setter(strip_option), default)]
    pub sdf_data: Option<HashMap<String, SdfData>>,
    #[builder(setter(strip_option), default)]
    pub sdf_input_data: Option<SdfData>,
    #[builder(setter(strip_option), default)]
    pub sdf_output_data: Option<SdfData>,

    #[builder(default)]
    pub sdf_protocol_map: Option<ActionProtocolMap>,

    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<HashMap<String, Value>>,
}

impl super::SdfAffordance for SdfAction {
    fn supported_uri_schemes(&self, sdf_operation: SdfOperation) -> anyhow::Result<Vec<String>> {
        self.sdf_protocol_map
            .as_ref()
            .context("Missing SDF protocol map.")?
            .supported_uri_schemes(sdf_operation)
    }
}

impl GlobalNameContributor for SdfAction {
    const QUALITY_NAME: &'static str = "sdfAction";
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
pub struct ActionProtocolMap {
    pub coap: Option<ActionCoapProtocolMap>,
    pub http: Option<ActionHttpProtocolMap>,
}

impl ActionProtocolMap {
    pub fn supported_uri_schemes(
        &self,
        _sdf_operation: SdfOperation,
    ) -> anyhow::Result<Vec<String>> {
        todo!()
    }
}
