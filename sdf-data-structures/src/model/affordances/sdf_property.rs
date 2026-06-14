use crate::{
    model::{affordances::SdfOperation, sdf_data::SdfData},
    traits::GlobalNameContributor,
    util::{default_bool_true, skip_bool_true},
};
use anyhow::Context;
use derive_builder::Builder;
use ploidy_pointer::{JsonPointee, JsonPointerTarget};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::model::protocol_mappings::{
    coap::PropertyCoapProtocolMap, http::PropertyHttpProtocolMap,
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
#[ploidy(pointer(rename_all = "camelCase"))]
pub struct SdfProperty {
    #[serde(flatten)]
    #[builder(default)]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub internal_data: SdfData,

    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub readable: bool,
    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub writable: bool,
    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub observable: bool,

    #[builder(default)]
    pub sdf_protocol_map: Option<PropertyProtocolMap>,
}

impl GlobalNameContributor for SdfProperty {
    const QUALITY_NAME: &'static str = "sdfProperty";
}

impl super::SdfAffordance for SdfProperty {
    fn supported_uri_schemes(&self, sdf_operation: SdfOperation) -> anyhow::Result<Vec<String>> {
        self.sdf_protocol_map
            .as_ref()
            .context("Missing SDF protocol map.")?
            .supported_uri_schemes(sdf_operation)
    }
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
pub struct PropertyProtocolMap {
    pub coap: Option<PropertyCoapProtocolMap>,
    pub http: Option<PropertyHttpProtocolMap>,
}

impl PropertyProtocolMap {
    fn supported_uri_schemes(&self, _sdf_operation: SdfOperation) -> anyhow::Result<Vec<String>> {
        // TODO: Refactor
        if let Some(coap_protocol_map) = &self.coap {
            return Ok(coap_protocol_map
                .sdf_operations
                .clone()
                .unwrap_or_default()
                .read
                .unwrap()
                .protocol_map
                .uri_schemes);
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sdf_property() {
        let sdf_property = SdfPropertyBuilder::default()
            .writable(false)
            .build()
            .unwrap();

        let serialized_sdf_property = "{\"writable\":false}".to_string();

        assert_eq!(
            serde_json::to_string(&sdf_property).unwrap(),
            serialized_sdf_property
        );
    }
}
