use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdfDataStructureError {
    #[error("resolving the target namespace failed: {0}")]
    TargetNamespaceError(String),
}
