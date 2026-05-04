// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use serde::Serialize;

use crate::error::SdfRepositoryError;

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct SemanticVersion {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) patch: u16,
}

impl From<SemanticVersion> for Vec<u16> {
    fn from(val: SemanticVersion) -> Self {
        vec![val.major, val.minor, val.patch]
    }
}

impl From<SemanticVersion> for Vec<i32> {
    fn from(val: SemanticVersion) -> Self {
        vec![val.major.into(), val.minor.into(), val.patch.into()]
    }
}

impl From<SemanticVersion> for String {
    fn from(val: SemanticVersion) -> Self {
        format!("{}.{}.{}", val.major, val.minor, val.patch)
    }
}

impl TryFrom<Vec<u16>> for SemanticVersion {
    type Error = SdfRepositoryError;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        let mut iterator = value.into_iter();

        let major = iterator.next().ok_or(SdfRepositoryError::InputParameters(
            "Invalid first sematic version component".to_string(),
        ))?;
        let minor = iterator.next().ok_or(SdfRepositoryError::InputParameters(
            "Invalid second sematic version component".to_string(),
        ))?;
        let patch = iterator.next().ok_or(SdfRepositoryError::InputParameters(
            "Invalid third sematic version component".to_string(),
        ))?;

        if iterator.next().is_some() {
            return Err(SdfRepositoryError::InputParameters(
                "Unexpected fourth version element".to_string(),
            ));
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl TryFrom<String> for SemanticVersion {
    type Error = SdfRepositoryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split_versions = value.split(".");

        let version_numbers: Result<Vec<_>, _> = split_versions
            .into_iter()
            .map(|x| x.parse::<u16>())
            .collect();

        version_numbers
            .map_err(|x| SdfRepositoryError::InputParameters(x.to_string()))?
            .try_into()
    }
}
