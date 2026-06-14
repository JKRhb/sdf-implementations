use sdf_data_structures::instance::SdfMessage;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) enum Unit {
    /// Celcius
    Cel,

    /// Fahrenheit
    F,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Cel => f.write_str("Cel"),
            Unit::F => f.write_str("F"),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ConfigurationData {
    pub(crate) device_name: Option<String>,
    pub(crate) unit: Option<Unit>,
}

impl<'a> TryFrom<&'a [u8]> for ConfigurationData {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let sdf_message = serde_json::from_slice::<SdfMessage>(value)?;

        let context_definitions = sdf_message.sdf_instance.sdf_context;

        if let Some(context_definitions) = context_definitions {
            let context_definition_value = serde_json::to_value(context_definitions)?;

            let configuration_data = serde_json::from_value::<Self>(context_definition_value)?;

            return Ok(configuration_data);
        }

        Err(anyhow::Error::msg(
            "Failed to deserialize ConfigurationData struct.",
        ))
    }
}
