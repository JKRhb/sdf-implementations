pub mod sdf_action;
pub mod sdf_event;
pub mod sdf_property;

#[derive(PartialEq, Debug)]
pub enum SdfOperation {
    Read,
    Write,
    Observe,
    Invoke,
    Subscribe,
}

pub trait SdfAffordance {
    fn supported_uri_schemes(&self, sdf_operation: SdfOperation) -> anyhow::Result<Vec<String>>;
}
