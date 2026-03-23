use sdf_data_structures::model::SchemaDefinition;

/// This tests the deserialization of the simple SDF example document contained
/// in [Figure 1] of the [SDF specification].
///
/// [Figure 1]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25#figure-1
/// [SDF specification]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25
#[test]
fn test_example1() {
    let input = "
    {
        \"info\": {
            \"title\": \"Example document for SDF (Semantic Definition Format)\",
            \"version\": \"2019-04-24\",
            \"copyright\": \"Copyright 2019 Example Corp. All rights reserved.\",
            \"license\": \"https://example.com/license\"
        },
        \"namespace\": {
            \"cap\": \"https://example.com/capability/cap\"
        },
        \"defaultNamespace\": \"cap\",
        \"sdfObject\": {
            \"Switch\": {
                \"sdfProperty\": {
                    \"value\": {
                        \"description\": \"The state of the switch; false for off and true for on.\",
                        \"type\": \"boolean\"
                    }
                },
                \"sdfAction\": {
                    \"on\": {
                        \"description\": \"Turn the switch on; equivalent to setting value to true.\"
                    },
                    \"off\": {
                        \"description\": \"Turn the switch off; equivalent to setting value to false.\"
                    },
                    \"toggle\": {
                        \"description\": \"Toggle the switch; equivalent to setting value to its complement.\"
                    }
                }
           }
        }
    }";

    let deserialized_input =
        serde_json::from_str::<sdf_data_structures::model::SdfModel>(input).unwrap();

    let info_block = deserialized_input.info.unwrap();

    assert_eq!(info_block.comment, Option::None,);
    assert_eq!(
        info_block.title,
        Option::Some("Example document for SDF (Semantic Definition Format)".into()),
    );
    assert_eq!(info_block.description, Option::None,);
    assert_eq!(info_block.version, Option::Some("2019-04-24".into()),);
    assert_eq!(
        info_block.copyright,
        Option::Some("Copyright 2019 Example Corp. All rights reserved.".into()),
    );
    assert_eq!(
        info_block.license,
        Option::Some("https://example.com/license".into()),
    );

    // TODO: Cover the rest of the example
}

/// This tests the deserialization of the example `sdfObject` definition
/// contained in [Figure 3] of the [SDF specification].
///
/// [Figure 3]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25#exobject
/// [SDF specification]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25
#[test]
fn test_sdf_object_definition() {
    let input = "
    {
        \"sdfObject\": {
            \"foo\": {
                \"sdfProperty\": {
                    \"bar\": {
                        \"type\": \"string\",
                        \"format\": \"foo\"
                    }
                }
            }
        }
    }";

    let deserialized_input =
        serde_json::from_str::<sdf_data_structures::model::SdfModel>(input).unwrap();

    let sdf_objects = deserialized_input.sdf_object.unwrap();

    let sdf_object = sdf_objects.get("foo").unwrap();

    let sdf_properties = sdf_object.sdf_property.as_ref().unwrap();

    let sdf_property = sdf_properties.get("bar").unwrap();

    let internal_data = &sdf_property.internal_data;

    match internal_data.r#type.as_ref().unwrap() {
        SchemaDefinition::String(string_schema) => {
            assert_eq!(string_schema.format.as_ref().unwrap(), "foo");
        }
        _ => panic!(),
    }
}

/// This tests the deserialization of the outlet strip example
/// contained in [Figure 7] of the [SDF specification].
///
/// [Figure 7]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25#figure-7
/// [SDF specification]: https://datatracker.ietf.org/doc/html/draft-ietf-asdf-sdf-25
#[test]
fn test_outlet_strip_example() {
    let input = "
    {
        \"sdfThing\": {
            \"outlet-strip\": {
                \"label\": \"Outlet strip\",
                \"description\": \"Contains a number of Sockets\",
                \"sdfObject\": {
                    \"socket\": {
                        \"description\": \"An array of sockets in the outlet strip\",
                        \"minItems\": 2,
                        \"maxItems\": 10
                    }
                }
            }
        }
    }";

    let deserialized_input =
        serde_json::from_str::<sdf_data_structures::model::SdfModel>(input).unwrap();

    let sdf_things = deserialized_input.sdf_thing.unwrap();

    let sdf_thing = sdf_things.get("outlet-strip").unwrap();
    let common_qualities = &sdf_thing.common_qualities;

    assert_eq!(common_qualities.label.as_ref().unwrap(), "Outlet strip");
    assert_eq!(
        common_qualities.description.as_ref().unwrap(),
        "Contains a number of Sockets"
    );

    let sdf_objects = sdf_thing.sdf_object.as_ref().unwrap();

    let sdf_object = sdf_objects.get("socket").unwrap();

    assert_eq!(
        sdf_object.common_qualities.description.as_ref().unwrap(),
        "An array of sockets in the outlet strip"
    );
    assert_eq!(*sdf_object.min_items.as_ref().unwrap(), 2);
    assert_eq!(*sdf_object.max_items.as_ref().unwrap(), 10);
}

// TODO: Also include test cases for the remaining examples
