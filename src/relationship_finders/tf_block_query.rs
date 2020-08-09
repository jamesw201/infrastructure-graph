/// tf_block_query 
/// Inputs:
/// - a limited jmespath string       *permissable jmespath syntax characters are '.' and '[]'
/// - a TerraformBlock
/// 
/// The entity will use the jmespath string to locate values in the TerraformBlock. It should be able to reach into any depth, including into nested json values.
/// 

use crate::json::{parse_json};

use crate::structs::json::JsonValue;

use crate::structs::template_string::{ TemplateString, BuiltInFunction };
use crate::structs::attributes::{ Attribute, AttributeType };
use AttributeType::{
    Array, Block, Boolean, Json, Num, Str, TFBlock, TemplatedString,
};

use crate::structs::terraform_block::{
    TerraformBlockWithTwoIdentifiers,
};

mod tf_block_query {
    use super::{ Attribute, AttributeType, TerraformBlockWithTwoIdentifiers };
    use super::JsonValue;

    #[derive(PartialEq, Debug, Clone)]
    pub enum PathPart {
        List(String),
        Scalar(String),
    }
    // pub struct PathPart {
    //     pub part: String,
    //     pub is_array: bool,
    // }

    #[derive(PartialEq, Debug, Clone)]
    pub struct JmespathExpression {
        pub path_parts: Vec<PathPart>,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub enum TFQueryResult {
        List(Vec<AttributeType>),
        Scalar(AttributeType),
        None,
    }

    pub fn parse_jmespath(jmespath_expression: &str) -> JmespathExpression {
        let dot_split = jmespath_expression.split(".").collect::<Vec<&str>>();

        let path_parts: Vec<PathPart> = dot_split.into_iter().map(|expr_part| {
            if expr_part.ends_with("[]") {
                let brackets_removed = expr_part[..expr_part.len()-2].to_string();
                
                PathPart::List(brackets_removed)
            } else {
                PathPart::Scalar(expr_part.to_string())
            }
        }).collect();

        JmespathExpression { path_parts }
    }

    fn find_attribute(attrs: Vec<AttributeType>) -> Option<AttributeType> {
        attrs.into_iter().find(|value| {
            match value {
                AttributeType::Json(json) => {
                    println!("found Json: {:?}", json);
                    false
                },
                attr => {
                    println!("found Attribute: {:?}", attr);
                    true
                },
            }
        })
    }

    pub fn recurse(path_parts: Vec<PathPart>, attrs: Vec<AttributeType>) -> Option<AttributeType> {
        println!("path_parts: {:?}", path_parts[0]);
        if path_parts.len() == 1 {
            println!("path_part: {:?}", path_parts[0]);
            find_attribute(attrs)
        } else {
            let found = find_attribute(attrs);
            match found {
                Some(attr_type) => recurse(path_parts[1..].to_vec(), vec![attr_type]),
                None => None
            }
        }
    }

    /// traverse a tf_block given a jmespath expression
    pub fn jmespath_query(tf_block: &TerraformBlockWithTwoIdentifiers, jmespath_expression: &str) -> TFQueryResult {
        let expression = parse_jmespath(jmespath_expression);
        println!("expression: {:?}", expression);

        // TODO: Check if path_parts.len() == 1 and any Attributes match

        // The point is to search through data structures for given attributes
        let attr = AttributeType::Str(String::from("sandbox1"));
        TFQueryResult::Scalar(attr)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::tf_block_query::*;

    fn example_resource() -> TerraformBlockWithTwoIdentifiers {
        TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_iam_role_policy"),
            second_identifier: String::from("discovery_scheduler_role_policy"),
            attributes: vec![
                Attribute {
                    key: String::from("depends_on"), value: Array(vec![AttributeType::Str(String::from("aws_iam_role.discovery_scheduler_role"))])
                },
                Attribute {
                    key: String::from("policy"),
                    value: Json(JsonValue::Object(vec![
                        (String::from("Statement"), JsonValue::Array(vec![
                            JsonValue::Object(vec![
                                (String::from("Action"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("logs:CreateLogStream")), JsonValue::Str(String::from("logs:CreateLogGroup")), JsonValue::Str(String::from("logs:PutLogEvents"))
                                ])),
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Resource"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_scheduler*"))
                                ]))
                            ]),
                            JsonValue::Object(vec![
                                (String::from("Action"), JsonValue::Array(vec![JsonValue::Str(String::from("dynamodb:Scan"))])),
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Resource"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config/*")),
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config")),
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")),
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))
                                ]))
                            ]),
                            JsonValue::Object(vec![
                                (String::from("Action"), JsonValue::Array(vec![JsonValue::Str(String::from("dynamodb:UpdateItem"))])),
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Resource"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")),
                                    JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))
                                ]))
                            ]),
                            JsonValue::Object(vec![
                                (String::from("Action"), JsonValue::Array(vec![JsonValue::Str(String::from("sns:Publish"))])),
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Resource"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic"))
                                ]))
                            ]),
                            JsonValue::Object(vec![
                                (String::from("Action"), JsonValue::Array(vec![JsonValue::Str(String::from("events:DescribeRule"))])),
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Resource"), JsonValue::Array(vec![
                                    JsonValue::Str(String::from("arn:aws:events:us-east-1:309983114184:rule/discovery_scheduler-rule"))
                                ]))
                            ])
                        ])),
                        (String::from("Version"), JsonValue::Str(String::from("2012-10-17")))
                    ]))
                },
                Attribute {
                    key: String::from("role"), value: TemplatedString(TemplateString::Variable(String::from("aws_iam_role.discovery_scheduler_role.id")))
                },
                Attribute {
                    key: String::from("name"), value: AttributeType::Str(String::from("discovery_scheduler_role_policy"))
                }
            ]
        }
    }

    #[test]
    fn recursion() {
        let data = vec![
            AttributeType::Str(String::from("some attribute")),
            AttributeType::Json(JsonValue::Str(String::from("json root attribute"))),
            AttributeType::Json(JsonValue::Str(String::from("uninteresting attribute"))),
        ];
        let path_parts = vec![
            PathPart::Scalar(String::from("policy")),
            PathPart::List(String::from("Statement")),
            PathPart::Scalar(String::from("resource")),
        ];
        // let result = tf_block_query::recurse(path_parts, example_resource().attributes);
        let expected = Some(AttributeType::Json(JsonValue::Str(String::from("bongo"))));
        // assert_eq!(result, expected)
        assert_eq!(1, 2)
    }

    #[test]
    fn parse_jmespath_expression() {
        let result = tf_block_query::parse_jmespath("policy.Statement[].resource");

        assert_eq!(result, JmespathExpression { path_parts: vec![
            PathPart::Scalar(String::from("policy")),
            PathPart::List(String::from("Statement")),
            PathPart::Scalar(String::from("resource")),
        ]})
    }

    #[test]
    fn query_tf_block() {
        let resource = example_resource();

        let result = tf_block_query::jmespath_query(&resource, "policy.Statement[].Resource");

        let expected = tf_block_query::TFQueryResult::List(
            vec![
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_scheduler*"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config/*"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic"))),
                AttributeType::Json(JsonValue::Str(String::from("arn:aws:events:us-east-1:309983114184:rule/discovery_scheduler-rule"))),
            ]
        );
        assert_eq!(result, expected)
    }

    // #[test]
    // fn deduplicate_resources() {
    //     //    "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key"
    //     //    and
    //     //    "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key/*"
    //     // s
    //     //    should result in a single relationship.
    //     let result = tf_block_query::parse_jmespath("policy.Statement[].resource");

    //     assert_eq!(1,2)
    // }
}
