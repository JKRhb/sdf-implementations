// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::model::SdfModel;
use serde_json::json;

use crate::{config::Config, error::SdfRepositoryError};

// TODO: Add second initial model
pub(crate) fn create_initial_models(config: &Config) -> Result<Vec<SdfModel>, SdfRepositoryError> {
    let mut namespace_url = config.get_base_url();

    namespace_url.push_str("/sdf/sensor");

    let first_initial_model = serde_json::from_value::<SdfModel>(json!({
        "info": {
            "lineage": "foobar",
            "version": "1.1.0"
        },
        "namespace": {
            "sensors": namespace_url
        },
        "defaultNamespace": "sensors",
        "sdfObject": {
            "envSensor": {
                "sdfContext": {
                    "ipAdress": {
                        "type": "string"
                    },
                    "deviceName": {
                        "type": "string"
                    },
                    "unit": {
                        "type": "string"
                    }
                },
                "sdfProperty": {
                    "temperature": {
                        "type": "string",
                        "sdfProtocolMap": {
                            "coap": {
                                "sdfParameters": {
                                    "ipAddress": "#/sdfObject/envSensor/sdfContext/ipAddress"
                                },
                                "sdfOperations": {
                                    "read": {
                                        "method": "GET",
                                        "href": "/temperature",
                                        "contentType": [60],
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }))?;

    Ok(vec![first_initial_model])
}
