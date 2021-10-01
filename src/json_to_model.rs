use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::structs::terraform_block::{
    TerraformBlock,
    TerraformBlock::{
        NoIdentifiers,
        WithOneIdentifier,
        WithTwoIdentifiers,
    },
    TerraformBlockWithNoIdentifiers,
    TerraformBlockWithOneIdentifier,
    TerraformBlockWithTwoIdentifiers,
};

use crate::structs::attributes::{ Attribute, AttributeType };
use AttributeType::{
    Array, Block, Boolean, Num, Str, TFBlock, TemplatedString,
};

pub fn convert_json_file_to_model(path: &str) -> bool {
    // let expected: Value = serde_json::from_str(input).unwrap();
    true
}

fn json_object_has_keys(json_str: &str, keys: Vec<&str>) -> bool {
    let expected: HashMap<&str, Value> = serde_json::from_str(json_str).unwrap();
    let object_keys: HashSet<&str> = expected.keys().cloned().collect();
    
    let hash_set: HashSet<&str> = keys.into_iter().collect();

    // TODO: match on the Value returned by serde and each of its sub Values to convert to a TerraformBLockWithTwoIdentifiers

    // match expected {

    // }
    
    object_keys == hash_set
}

pub fn convert_json_string_to_model(input: &str) -> bool {
    let keys = vec!["name", "type", "body"];
    let matching = json_object_has_keys(input, keys);

    println!("contains: {:?}", matching);
    // println!("expected: {:?}", expected);
    true
}

fn match_value(value: &Value) -> AttributeType {
    match value {
        Value::Null => AttributeType::Str(String::from("null")),
        Value::String(val) => AttributeType::Str(val.clone()),
        Value::Number(val) => AttributeType::Num(val.as_f64().unwrap_or_default()),
        Value::Bool(val) => AttributeType::Boolean(val.clone()),
        Value::Array(vals) => AttributeType::Array(vals.into_iter().map(|val| match_value(val)).collect()),
        Value::Object(vals) => {
            let block_attributes = vals.into_iter().map(|(key, val)| Attribute { key: key.clone(), value: match_value(val) }).collect();
            AttributeType::Block(block_attributes)
        },
    }
}

pub fn make_block_with_two_identifiers_from_json(input: &str) -> TerraformBlockWithTwoIdentifiers {
    let serde_output: HashMap<&str, Value> = serde_json::from_str(input).unwrap();
    let first_identifier: String = Deserialize::deserialize(serde_output.get("type").unwrap()).unwrap();
    let second_identifier: String = Deserialize::deserialize(serde_output.get("name").unwrap()).unwrap();
    let body = serde_output.get("body").unwrap_or(&Value::Null);
    let body_attributetype = match_value(body);

    let attributes: Vec<Attribute> = match body_attributetype {
        AttributeType::Block(attrs) => attrs,
        _ => vec![],
    };

    TerraformBlockWithTwoIdentifiers {
        block_type: String::from("resource"),
        first_identifier,
        second_identifier,
        attributes,
    }
}

pub fn convert_json_to_model(input: &str) -> TerraformBlock {
    let block = make_block_with_two_identifiers_from_json(input);
    TerraformBlock::WithTwoIdentifiers(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_json() -> &'static str {
        r#"{
            "type": "aws_kms_key",
            "name": "discovery_cache-master-key",
            "body": {
                "description": "Master key used for creating/decrypting cache token data keys",
                "enable_key_rotation": "true",
                "lifecycle": {
                    "prevent_destroy": "true"
                }
            }
        }"#
    }

    fn setup_terraform_block() -> TerraformBlock {
        TerraformBlock::WithTwoIdentifiers(
            TerraformBlockWithTwoIdentifiers {
                block_type: String::from("resource"),
                first_identifier: String::from("aws_kms_key"),
                second_identifier: String::from("discovery_cache-master-key"),
                attributes: vec![
                    Attribute {
                        key: String::from("description"),
                        value: Str(String::from("Master key used for creating/decrypting cache token data keys"))
                    },
                    Attribute {
                        key: String::from("enable_key_rotation"),
                        value: Str(String::from("true"))
                    },
                    Attribute {
                        key: String::from("lifecycle"),
                        value: Block(
                            vec![
                                Attribute {
                                    key: String::from("prevent_destroy"),
                                    value: Str(String::from("true"))
                                },
                            ]
                        )
                    }
                ]
            }
        )
    }

    #[test]
    fn convert_by_matching() {
        let data = setup_json();
        let expected = setup_terraform_block();

        let result = super::convert_json_to_model(data);
        assert_eq!(result, expected)
    }

    // #[test]
    // fn convert_json_file_to_model() {
    // }
}   