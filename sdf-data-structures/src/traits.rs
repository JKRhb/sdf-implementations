// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};

use anyhow::{Context, bail};

use crate::{
    error::SdfDataStructureError,
    model::{SdfAction, SdfContext, SdfEvent, SdfObject, SdfProperty, SdfThing},
};

pub trait GlobalNameContributor {
    const QUALITY_NAME: &'static str;

    fn get_global_name(&self, prefix: &String, result: &mut HashSet<String>, given_name: &String) {
        let global_name = format!("{prefix}/{}/{given_name}", Self::QUALITY_NAME);
        result.insert(global_name);
    }
}

// TODO: Turn into trait
#[derive(Clone, Debug)]
pub enum SdfGrouping {
    SdfObject(SdfObject),
    SdfThing(SdfThing),
}

impl SdfGrouping {
    pub fn sdf_property(self) -> Option<HashMap<String, SdfProperty>> {
        match self {
            SdfGrouping::SdfObject(sdf_object) => sdf_object.sdf_property,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_property,
        }
    }

    pub fn sdf_action(self) -> Option<HashMap<String, SdfAction>> {
        match self {
            SdfGrouping::SdfObject(sdf_object) => sdf_object.sdf_action,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_action,
        }
    }

    pub fn sdf_event(self) -> Option<HashMap<String, SdfEvent>> {
        match self {
            SdfGrouping::SdfObject(sdf_object) => sdf_object.sdf_event,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_event,
        }
    }

    pub fn sdf_context(self) -> Option<HashMap<String, SdfContext>> {
        match self {
            SdfGrouping::SdfObject(sdf_object) => sdf_object.sdf_context,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_context,
        }
    }

    pub fn sdf_thing(self) -> Option<HashMap<String, SdfThing>> {
        match self {
            SdfGrouping::SdfObject(_sdf_object) => None,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_thing,
        }
    }

    pub fn sdf_object(self) -> Option<HashMap<String, SdfObject>> {
        match self {
            SdfGrouping::SdfObject(_sdf_object) => None,
            SdfGrouping::SdfThing(sdf_thing) => sdf_thing.sdf_object,
        }
    }

    // TODO: Replace with functions that create maps with JSON pointers and all affordances
    pub fn resolve_affordance_pointer(
        self,
        affordance_pointer: &str,
    ) -> anyhow::Result<Option<SdfAffordance>> {
        let mut pointer_segments = affordance_pointer
            .trim_start_matches("#")
            .trim_start_matches("/")
            .split("/");

        let first_element = pointer_segments
            .next()
            .context("Missing pointer segment for sdf quality name")?;
        let second_element = pointer_segments
            .next()
            .context("Missing pointer segment for given quality name")?;

        let result = match first_element {
            "sdfProperty" => self
                .sdf_property()
                .context("Missing sdfProperty quality")?
                .get(second_element)
                .map(|x| SdfAffordance::SdfProperty(x.clone())),
            "sdfAction" => self
                .sdf_action()
                .context("Missing sdfAction quality")?
                .get(second_element)
                .map(|x| SdfAffordance::SdfAction(x.clone())),
            "sdfEvent" => self
                .sdf_event()
                .context("Missing sdfEvent quality")?
                .get(second_element)
                .map(|x| SdfAffordance::SdfEvent(x.clone())),
            // "sdfObject" => {
            //     self.sdf_object().context("hi")?.get(second_element).context("hey")?.
            // }
            _ => todo!(),
        };

        Ok(result)
    }
}

pub enum SdfAffordance {
    SdfProperty(SdfProperty),
    SdfAction(SdfAction),
    SdfEvent(SdfEvent),
}

pub trait SdfDataStructure {
    fn namespace(&self) -> Option<&HashMap<String, String>>;

    fn default_namespace(&self) -> Option<&String>;

    fn get_target_namespace(&self) -> anyhow::Result<Option<String>> {
        match self.default_namespace() {
            Some(default_namespace) => {
                if let Some(namespace) = self.namespace() {
                    if let Some(namespace_url) = namespace.get(default_namespace) {
                        return Ok(Some(namespace_url.to_string()));
                    }

                    bail!(SdfDataStructureError::TargetNamespaceError(
                        "Target namespace set, but no namespace map defined!.".to_string()
                    ))
                }

                bail!(SdfDataStructureError::TargetNamespaceError(
                    "Target namespace not in namespace map.".to_string()
                ))
            }
            None => Ok(None),
        }
    }
}

pub trait GlobalNameAggregator {
    fn determine_global_names(&self) -> HashSet<String>;
}
