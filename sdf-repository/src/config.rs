// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use dotenv_config::EnvConfig;

/// The configuration that is used by the SDF Repository.
#[derive(Debug, EnvConfig, Clone)]
pub(crate) struct Config {
    /// The IP address to bind to.
    #[env_config(default = "127.0.0.1")]
    pub(crate) bind_address: String,

    /// The TCP port to bind to.
    #[env_config(default = 8080)]
    pub(crate) port: u16,

    /// The URI scheme used in namespace URLs.
    #[env_config(default = "http")]
    pub(crate) namespace_uri_scheme: String,

    /// The hostname to be used in namespace URLs.
    #[env_config(default = "localhost")]
    pub(crate) hostname: String,

    /// Whether the port number should be included in the namespace URL.
    #[env_config(default = true)]
    pub(crate) include_port_in_namespace_url: bool,

    /// Whether resources that create, update, or delete models should be
    /// protected via basic authentication (username and password).
    #[env_config(default = true)]
    pub(crate) basic_auth_enabled: bool,

    /// The username to be used with basic authentication.
    ///
    /// Only relevant when basic authentication is enabled.
    #[env_config(default = "")]
    pub(crate) username: String,

    /// The password to be used with basic authentication.
    ///
    /// Only relevant when basic authentication is enabled.
    #[env_config(default = "")]
    pub(crate) password: String,
}

impl Config {
    /// Returns the base URL that is used for namespace definitions.
    pub(crate) fn get_base_url(&self) -> String {
        let base_url = format!("{}://{}", self.namespace_uri_scheme, self.hostname);

        if self.include_port_in_namespace_url {
            format!("{base_url}:{}", self.port)
        } else {
            base_url
        }
    }
}
