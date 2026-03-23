use std::collections::HashMap;

use anyhow::bail;

use crate::error::SdfDataStructureError;

pub trait SdfDataStructure {
    fn namespace(&self) -> Option<&HashMap<String, String>>;

    fn default_namespace(&self) -> Option<&String>;

    fn get_target_namespace(&self) -> anyhow::Result<Option<String>> {
        match self.default_namespace() {
            Some(default_namespace) => {
                if let Some(namespace) = self.namespace() {
                    if let Some(namespace_url) = namespace.get(default_namespace) {
                        return Ok(Some(namespace_url.to_string()));
                    }

                    bail!(SdfDataStructureError::TargetNamespaceError(
                        "Target namespace set, but no namespace map defined!.".to_string()
                    ))
                }

                bail!(SdfDataStructureError::TargetNamespaceError(
                    "Target namespace not in namespace map.".to_string()
                ))
            }
            None => Ok(None),
        }
    }
}
