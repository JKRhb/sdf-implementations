// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use actix_web::{Error, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::basic::BasicAuth;
use log::{info, warn};

use crate::persistence::AppState;

pub(crate) async fn basic_authentication_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> actix_web::Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_data = req
        .app_data::<web::Data<AppState>>()
        .expect("invalid app state");
    let config = &app_data.config;

    if !config.basic_auth_enabled {
        warn!("Skipping basic authentication as it is not enabled in the configuration!");
        return Ok(req);
    }

    if credentials.user_id() == config.username && credentials.password() == Some(&config.password)
    {
        info!("Basic authentication successful");
        Ok(req)
    } else {
        Err((
            ErrorUnauthorized("Registering, deleting, and updating models requires authorization"),
            req,
        ))
    }
}
