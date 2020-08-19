use serde::{Deserialize, Serialize};
use crate::structs::traits::query::Queryable;

use crate::structs::attributes::{ Attribute, AttributeType };
use crate::relationship_finders::tf_block_query::tf_block_query::{JmespathExpression, PathPart};
use PathPart::{ List, Scalar};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Null(String),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn convert_to_attribute_type(json_value: JsonValue) -> AttributeType {
        match json_value {
            Self::Str(value) => AttributeType::Str(value.to_string()),
            Self::Boolean(value) => AttributeType::Boolean(value.clone()),
            Self::Null(value) => AttributeType::Str(value.clone()),
            Self::Num(value) => AttributeType::Num(value.clone()),
            Self::Array(value) => AttributeType::Array(
                value.into_iter().map(|val| Self::convert_to_attribute_type(val)).collect()
            ),
            Self::Object(value) => AttributeType::Block(
                value.into_iter().map(|(key, val)| 
                    Attribute{ key: key.to_string(), value: Self::convert_to_attribute_type(val) } 
                ).collect()
            ),
        }
    }
}

impl Queryable for JsonValue {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType> {
        println!("JsonValue expression {:?}", expression);

        if expression.path_parts.len() == 1 {
            // println!("JsonValue {:?}", self);

            let queried_json_results = match self {
                JsonValue::Object(obj_vals) => {
                    let found = obj_vals.iter().find(|(key, _)| {
                        match &expression.path_parts[0] {
                            Scalar(path) => key == &path.to_string(),
                            List(path) => key == &path.to_string(),
                        }
                    });
                    if let Some(actual_value) = found {
                        vec![actual_value]
                    } else {
                        vec![]
                    }
                },
                JsonValue::Array(array_vals) => {
                    // if the first element is a JsonValue::Object then the rest will be too
                    array_vals.into_iter().map(|obj| {
                        match obj {
                            JsonValue::Object(obj_vals) => {
                                obj_vals.iter().find(|(key, _)| {
                                    match &expression.path_parts[0] {
                                        Scalar(path) => key == &path.to_string(),
                                        List(path) => key == &path.to_string(),
                                    }
                                })
                            },
                            _ => None,
                        }
                    }).flatten().collect()
                },
                _ => vec![]
            };
            
            let attributes = queried_json_results.into_iter().map(|(key, json_val)| {
                let clone = json_val.clone();
                Attribute { key: key.to_string(), value: Self::convert_to_attribute_type(clone) }
            }).collect();

            let attribute_array = AttributeType::Block(attributes);

            Some(attribute_array)
        } else {
            // println!("JsonValue attribute {:?}", &self);

            // if path_part is a List then the the target has to be a List also?
            match &expression.path_parts[0] {
                List(path) => {
                    if let JsonValue::Object(val) = self {
                        let res = val.into_iter()
                            .filter(|(key, _)| key == &path.to_string())
                            .map(|(_, json_val)| {
                                let new_expression = JmespathExpression { path_parts: expression.path_parts[1..].to_vec() };
                                json_val.query(new_expression)
                            }).flatten().collect();

                        let result = AttributeType::Array(res);
                        Some(result)
                    } else {
                        None    // return None because only an Object is applicable for a List path_part
                    }
                },
                Scalar(_) => {
                    println!("we're looking at an unhandled Scalar");
                    None
                },
            }
        }
    }
}
