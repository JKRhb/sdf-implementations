// Copyright 2026 Jan Romann
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: MIT

use sdf_data_structures::instance::InfoBlockBuilder;
use sdf_data_structures::instance::SdfInstanceBuilder;
use sdf_data_structures::instance::SdfInstanceOfBuilder;
use sdf_data_structures::instance::SdfMessageBuilder;
use sdf_data_structures::model::CommonQualitiesBuilder;
use sdf_data_structures::model::SdfModelBuilder;
use sdf_data_structures::model::SdfObjectBuilder;
use sdf_data_structures::traits::SdfDataStructure;
use std::collections::HashMap;

#[test]
fn test_sdf_object_definition() {
    let sdf_message = SdfMessageBuilder::default()
        .namespace(HashMap::from_iter(vec![(
            "foo".into(),
            "https://example.org/foo/foobar".into(),
        )]))
        .default_namespace("foo")
        .info(
            InfoBlockBuilder::default()
                .message_id("75532020-8f64-4daf-a241-fcb0b6dc4a85")
                .build()
                .unwrap(),
        )
        .sdf_instance_of(
            SdfInstanceOfBuilder::default()
                .entry_point("#/sdfObject/bar")
                .build()
                .unwrap(),
        )
        .sdf_instance(SdfInstanceBuilder::default().build().unwrap())
        .build()
        .unwrap();

    let target_namespace_url = sdf_message.get_target_namespace().unwrap();

    assert_eq!(
        target_namespace_url,
        Some("https://example.org/foo/foobar".to_string())
    );

    let sdf_object = SdfObjectBuilder::default()
        .common_qualities(
            CommonQualitiesBuilder::default()
                .comment("This is a test!")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    SdfModelBuilder::default()
        .sdf_object(HashMap::from([("bar".into(), sdf_object.clone())]))
        .build()
        .unwrap();
}
