use anyhow::Context;
use json_pointer::JsonPointer;
use serde_json::{Map, Value};

pub(super) fn obtain_operation(
    protocol_map: &Map<String, Value>,
    operation: String,
) -> anyhow::Result<&Map<String, Value>> {
    let http_property_operations = protocol_map
        .get("sdfOperations")
        .and_then(|x| x.as_object())
        .context("Missing sdfOperations in sdfProtocolMap definition.")?;

    http_property_operations
        .get(&operation)
        .and_then(|x| x.as_object())
        .context("HTTP-related sdfProtocolMap does not support the read operation")
}

pub(super) fn obtain_method(
    operation: &Map<String, Value>,
    default_method: &'static str,
) -> String {
    operation
        .get("method")
        .and_then(|x| x.as_str())
        .unwrap_or(default_method)
        .to_string()
}

pub(crate) fn obtain_entry_point(sdf_instance: &Value) -> anyhow::Result<String> {
    sdf_instance
        .get("sdfInstanceOf")
        .and_then(|x| x.get("entryPoint"))
        .context("Missing entryPoint quality.")?
        .as_str()
        .map(|x| x.to_string())
        .context("Wrong data type for entryPoint quality.")
}

pub(super) fn determine_url(
    operation: &Map<String, Value>,
    protocol_map: &Map<String, Value>,
    sdf_instance: &Value,
    _sdf_model: &Value,
    default_scheme: &'static str,
) -> anyhow::Result<String> {
    let href = operation
        .get("href")
        .and_then(|x| x.as_str())
        .context("Missing href")?;

    let scheme = operation
        .get("scheme")
        .and_then(|x| x.as_str())
        .unwrap_or(default_scheme);

    let ip_address_pointer = protocol_map
        .get("sdfParameters")
        .context("Missing sdfParameters definition.")?
        .get("ipAddress")
        .and_then(|x| x.as_str());

    let entry_point = obtain_entry_point(sdf_instance)?;

    let instance_host_json_pointer = ip_address_pointer
        // .or(ip_address_pointer)
        .context("hi")?
        .trim_start_matches(&entry_point);

    let host = format!("/sdfInstance{instance_host_json_pointer}")
        .parse::<JsonPointer<_, _>>()
        .unwrap()
        .get(sdf_instance)
        .map(|x| x.as_str())
        .unwrap()
        .unwrap();

    Ok(format!("{scheme}://{host}{href}"))
}
