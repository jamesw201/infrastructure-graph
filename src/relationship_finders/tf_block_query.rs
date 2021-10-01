/// tf_block_query 
/// Inputs:
/// - a limited jmespath string       *permissable jmespath syntax characters are '.' and '[]'
/// - a TerraformBlock
/// 
/// The entity will use the jmespath string to locate values in the TerraformBlock. It should be able to reach into any depth, including into nested json values.
/// 

use crate::json::{parse_json};
use crate::structs::traits::query::Queryable;

use crate::structs::template_string::{ TemplateString, BuiltInFunction };
use crate::structs::attributes::{ Attribute, AttributeType };
use AttributeType::{
    Array, Block, Boolean, Num, Str, TFBlock, TemplatedString,
};

use crate::structs::terraform_block::{
    TerraformBlockWithTwoIdentifiers,
};

pub mod tf_block_query {
    use super::{ Attribute, AttributeType, TerraformBlockWithTwoIdentifiers };
    // use super::JsonValue;
    use super::Queryable;

    #[derive(PartialEq, Debug, Clone)]
    pub enum QueryPart {
        List(String),
        Scalar(String),
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct JmespathExpression {
        pub path_parts: Vec<QueryPart>,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub enum TFQueryResult {
        List(Vec<AttributeType>),
        Scalar(AttributeType),
        None,
    }

    pub fn parse_jmespath(jmespath_expression: &str) -> JmespathExpression {
        let dot_split = jmespath_expression.split(".").collect::<Vec<&str>>();

        let path_parts: Vec<QueryPart> = dot_split.into_iter().map(|expr_part| {
            if expr_part.ends_with("[]") {
                let brackets_removed = expr_part[..expr_part.len()-2].to_string();
                
                QueryPart::List(brackets_removed)
            } else {
                QueryPart::Scalar(expr_part.to_string())
            }
        }).collect();

        JmespathExpression { path_parts }
    }

    /// traverse a tf_block given a jmespath expression
    pub fn jmespath_query(tf_block: &TerraformBlockWithTwoIdentifiers, jmespath_expression: &str) -> TFQueryResult {
        let expression = parse_jmespath(jmespath_expression);
        let result = tf_block.query(&expression);
        
        // TODO: this is a hack, find a better representation in the Type instead
        let is_list = expression.path_parts.iter().any(|part| {
            match part {
                QueryPart::List(_) => true,
                _ => false,
            }
        });

        if result.len() > 0 {
            if is_list {
                TFQueryResult::List(result.clone())
            } else {
                TFQueryResult::Scalar(result[0].clone())
            }
        } else {
            TFQueryResult::None
        }
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
                    value: AttributeType::Block(vec![
                        Attribute { key: String::from("Statement"), value: AttributeType::Array(vec![
                            AttributeType::Block(vec![
                                Attribute { key: String::from("Action"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("logs:CreateLogStream")), AttributeType::Str(String::from("logs:CreateLogGroup")), AttributeType::Str(String::from("logs:PutLogEvents"))
                                ])},
                                Attribute { key: String::from("Effect"), value: AttributeType::Str(String::from("Allow")) },
                                Attribute { key: String::from("Resource"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_scheduler*"))
                                ])}
                            ]),
                            AttributeType::Block(vec![
                                Attribute { key: String::from("Action"), value: AttributeType::Array(vec![AttributeType::Str(String::from("dynamodb:Scan"))])},
                                Attribute { key: String::from("Effect"), value: AttributeType::Str(String::from("Allow"))},
                                Attribute { key: String::from("Resource"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config/*")),
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config")),
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")),
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))
                                ])}
                            ]),
                            AttributeType::Block(vec![
                                Attribute { key: String::from("Action"), value: AttributeType::Array(vec![AttributeType::Str(String::from("dynamodb:UpdateItem"))])},
                                Attribute { key: String::from("Effect"), value: AttributeType::Str(String::from("Allow"))},
                                Attribute { key: String::from("Resource"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")),
                                    AttributeType::Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"))
                                ])}
                            ]),
                            AttributeType::Block(vec![
                                Attribute { key: String::from("Action"), value: AttributeType::Array(vec![AttributeType::Str(String::from("sns:Publish"))])},
                                Attribute { key: String::from("Effect"), value: AttributeType::Str(String::from("Allow"))},
                                Attribute { key: String::from("Resource"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic"))
                                ])}
                            ]),
                            AttributeType::Block(vec![
                                Attribute { key: String::from("Action"), value: AttributeType::Array(vec![AttributeType::Str(String::from("events:DescribeRule"))])},
                                Attribute { key: String::from("Effect"), value: AttributeType::Str(String::from("Allow"))},
                                Attribute { key: String::from("Resource"), value: AttributeType::Array(vec![
                                    AttributeType::Str(String::from("arn:aws:events:us-east-1:309983114184:rule/discovery_scheduler-rule"))
                                ])}
                            ])
                        ])},
                        Attribute { key: String::from("Version"), value: AttributeType::Str(String::from("2012-10-17")) }
                    ])
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
    fn parse_jmespath_expression() {
        let result = tf_block_query::parse_jmespath("policy.Statement[].resource");

        assert_eq!(result, JmespathExpression { path_parts: vec![
            QueryPart::Scalar(String::from("policy")),
            QueryPart::List(String::from("Statement")),
            QueryPart::Scalar(String::from("resource")),
        ]})
    }

    #[test]
    fn query_tf_block() {
        let resource = example_resource();

        let result = tf_block_query::jmespath_query(&resource, "policy.Statement[].Resource");

        let expected = tf_block_query::TFQueryResult::List(
            vec![
                Array(vec![Str(String::from("arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_scheduler*"))]), 
                Array(vec![
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config/*")), 
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_collector-config")), 
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")), 
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config")),
                ]), 
                Array(vec![
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*")), 
                    Str(String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config")),
                ]), 
                Array(vec![
                    Str(String::from("arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic")),
                ]), 
                Array(vec![Str(String::from("arn:aws:events:us-east-1:309983114184:rule/discovery_scheduler-rule"))]),
            ]
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn query_tf_block_for_single_root_attribute() {
        let resource = example_resource();

        let result = tf_block_query::jmespath_query(&resource, "depends_on");

        let expected = tf_block_query::TFQueryResult::Scalar(
            AttributeType::Array(
                vec![
                    AttributeType::Str(String::from("aws_iam_role.discovery_scheduler_role"))
                ]
            )
        );
        assert_eq!(result, expected)
    }
}
