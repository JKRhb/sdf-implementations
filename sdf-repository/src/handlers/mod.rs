// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;

use actix_web::guard::GuardContext;
use semver::Version;

use crate::error::SdfRepositoryError;

pub(crate) mod delete_models;
pub(crate) mod get_model;
pub(crate) mod get_models;
pub(crate) mod post_model;
pub(crate) mod post_supplement;



fn verify_content_type(ctx: &GuardContext, expected_content_type: &str) -> bool {
    let content_type_value = ctx.head().headers().get("content-type");

    if let Some(content_type_value) = content_type_value {
        return content_type_value
            .to_str()
            .is_ok_and(|content_type| content_type == expected_content_type);
    }

    false
}
