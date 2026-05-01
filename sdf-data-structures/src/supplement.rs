use std::collections::{HashMap, HashSet};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{
    traits::{GlobalNameAggregator, SdfDataStructure},
    util::{default_bool_true, none_extra, skip_bool_true},
};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SdfSupplement {
    #[builder(setter(strip_option), default)]
    pub info: Option<InfoBlock>,
    #[builder(setter(into, strip_option), default)]
    pub namespace: Option<HashMap<String, String>>,
    #[builder(setter(into, strip_option), default)]
    pub default_namespace: Option<String>,
    #[builder(setter(into), default)]
    pub amend: Vec<HashMap<String, Amendment>>,
}

impl SdfDataStructure for SdfSupplement {
    fn namespace(&self) -> Option<&HashMap<String, String>> {
        self.namespace.as_ref()
    }

    fn default_namespace(&self) -> Option<&String> {
        self.default_namespace.as_ref()
    }
}

impl SdfSupplement {
    /// Returns the default namespace URL from the `namespace` quality as indicated
    /// by the value of the `defaultNamespace` quality.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::supplement::SdfSupplementBuilder;
    /// use std::collections::HashMap;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfSupplementBuilder::default()
    ///     .namespace(HashMap::from_iter(vec![("foo".to_string(), "https://example.org".to_string())]))
    ///     .default_namespace("foo")
    ///     .build()?;
    ///
    /// assert_eq!(model.get_default_namespace_url(), Some("https://example.org".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_default_namespace_url(&self) -> Option<String> {
        self.namespace
            .clone()?
            .get(&self.default_namespace.clone()?)
            .cloned()
    }

    /// Returns the value of the `version` quality within this supplement's `info` block, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::supplement::SdfSupplementBuilder;
    /// use sdf_data_structures::supplement::InfoBlockBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfSupplementBuilder::default()
    ///     .info(InfoBlockBuilder::default().version("1.0.0").build()?)
    ///     .build()?;
    ///
    /// assert_eq!(model.get_version(), Some("1.0.0".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_version(&self) -> Option<String> {
        self.info.as_ref().and_then(|info| info.version.clone())
    }

    /// Returns the value of the `targetVersion` quality within this supplement's `info` block, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::supplement::SdfSupplementBuilder;
    /// use sdf_data_structures::supplement::InfoBlockBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfSupplementBuilder::default()
    ///     .info(InfoBlockBuilder::default().target_version("1.0.0").build()?)
    ///     .build()?;
    ///
    /// assert_eq!(model.get_target_version(), Some("1.0.0".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_target_version(&self) -> Option<String> {
        self.info
            .as_ref()
            .and_then(|info| info.target_version.clone())
    }

    /// Returns the value of the `lineage` quality within this supplement's `info` block, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use sdf_data_structures::supplement::SdfSupplementBuilder;
    /// use sdf_data_structures::supplement::InfoBlockBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// #
    /// let model = SdfSupplementBuilder::default()
    ///     .info(InfoBlockBuilder::default().lineage("foobar").build()?)
    ///     .build()?;
    ///
    /// assert_eq!(model.get_lineage(), Some("foobar".to_string()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_lineage(&self) -> Option<String> {
        self.info.as_ref().and_then(|info| info.lineage.clone())
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum PatchMethod {
    #[default]
    MergePatch,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Amendment {
    pub delta: Value,

    #[builder(setter(strip_option), default = "true")]
    #[serde(default = "default_bool_true", skip_serializing_if = "skip_bool_true")]
    pub fix: bool,

    #[builder(setter(strip_option), default = "PatchMethod::MergePatch")]
    #[serde(default)]
    pub patch_method: PatchMethod,
}

#[skip_serializing_none]
#[derive(PartialEq, Default, Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct InfoBlock {
    #[builder(setter(into, strip_option), default)]
    pub title: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub lineage: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub target_version: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub modified: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub copyright: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub license: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub timestamp: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub features: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    #[serde(rename = "$comment")]
    pub comment: Option<String>,

    #[serde(flatten, deserialize_with = "none_extra")]
    #[builder(setter(into, strip_option), default)]
    pub additional_qualities: Option<Map<String, Value>>,
}

impl GlobalNameAggregator for SdfSupplement {
    fn determine_global_names(&self) -> HashSet<String> {
        let namespace_url = self.get_default_namespace_url();

        if let Some(namespace_url) = namespace_url {
            let global_names = self
                .amend
                .iter()
                .flat_map(|x| x.keys().map(|key| format!("{}{}", namespace_url, key)))
                .collect::<Vec<_>>();

            HashSet::from_iter(global_names)
        } else {
            HashSet::new()
        }
    }
}
