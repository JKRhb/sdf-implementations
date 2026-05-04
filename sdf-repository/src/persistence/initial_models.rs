// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement};
use serde_json::json;

use crate::{error::SdfRepositoryError, models::config::Config};

pub(crate) fn create_initial_model(config: &Config) -> Result<SdfModel, SdfRepositoryError> {
    let mut namespace_url = config.get_base_url();

    namespace_url.push_str("/sdf/sensor");

    let sdf_model = serde_json::from_value::<SdfModel>(json!({
        "info": {
            "lineage": "foobar",
            "version": "1.0.0"
        },
        "namespace": {
            "sensors": namespace_url
        },
        "defaultNamespace": "sensors",
        "sdfObject": {
            "envSensor": {
                "sdfContext": {
                    "ipAddress": {
                        "type": "string"
                    },
                    "unit": {
                        "type": "string"
                    }
                },
                "sdfProperty": {
                    "temperature": {
                        "type": "number",
                        "sdfProtocolMap": {
                            "coap": {
                                "sdfParameters": {
                                    "ipAddress": "#/sdfObject/envSensor/sdfContext/ipAddress"
                                },
                                "sdfOperations": {
                                    "read": {
                                        "method": "GET",
                                        "href": "/temperature",
                                        "contentType": [60]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }))?;

    Ok(sdf_model)
}

pub(crate) fn create_initial_supplement(
    config: &Config,
) -> Result<SdfSupplement, SdfRepositoryError> {
    let mut namespace_url = config.get_base_url();

    namespace_url.push_str("/sdf/sensor");

    let sdf_supplement = serde_json::from_value::<SdfSupplement>(json!(
        {
            "info": {
                "lineage": "foobar",
                "targetVersion": "1.0.0"
            },
            "namespace": {
                "sensors": namespace_url

            },
            "defaultNamespace": "sensors",
            "amend": [
                {
                    "#/sdfObject/envSensor/sdfContext": {
                        "delta": {
                            "deviceName": {
                                "type": "string"
                            },
                        }
                    }
                }
            ]
        }
    ))?;

    Ok(sdf_supplement)
}
