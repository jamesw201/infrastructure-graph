use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::structs::terraform_block::TerraformBlockWithTwoIdentifiers;
use crate::relationship_finders::tf_block_query::tf_block_query::{ jmespath_query, TFQueryResult };

use TFQueryResult::{ Scalar, List };

#[derive(Debug)]
pub struct ValidationError {
    details: String
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError{details: msg.to_string()}
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub label: String,
}

impl Relationship {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.target != "policy.Statement[].Resource" {
            // the value of the username will automatically be added later
            return Err(ValidationError::new("target is not formatted correctly"));
        }
    
        Ok(())
    }
}

impl std::fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"{{"in":"{}","out":"{}","label":"{}"}}"#, self.source, self.target, self.label)
    }
}

pub fn convert_arn_to_dot_syntax(arn_resource: &String) -> Option<String> {
    let tokens: Vec<&str> = arn_resource.split(":").collect();

    let dot_syntax_resource = match tokens[2] {
        "dynamodb" => Some(format!("aws_dynamodb_table.{}", tokens[5].replace("table/", "").replace("/*", ""))),
        "sns" => Some(format!("aws_sns_topic.{}", tokens[5])),
        "s3" => {
            // let first_slash = tokens[5].find("/");

            // match first_slash {
            //     Some(position) => Some(format!("aws_s3_bucket.{}", &tokens[5][0..position])),
            //     None => Some(format!("aws_s3_bucket.{}", tokens[5])),
            // }
            if tokens[5].ends_with("/*") {
                let without_wildcard = tokens[5].len() - 2;
                Some(format!("aws_s3_bucket.{}", &tokens[5][0..without_wildcard]))
            } else {
                Some(format!("aws_s3_bucket.{}", tokens[5]))
            }
        },
        "lambda" => Some(format!("aws_lambda_function.{}", tokens[6])),
        "sqs" => Some(format!("aws_sqs_queue.{}", tokens[5])),
        "kinesis" => {
            let first_slash = tokens[5].find("/");

            match first_slash {
                Some(position) => Some(format!("aws_kinesis_stream.{}", &tokens[5][0..position])),
                None => Some(format!("aws_kinesis_stream.{}", &tokens[5])),
            }
        },
        "logs" => None, // log relationships are not wanted on a diagram
        anything => {
            println!("unknown arn: {:?}, for resource: {}", anything, arn_resource);
            None
        },
    };

    dot_syntax_resource
}

fn clean_resource_identifier(resource: &String) -> Option<String> {
    if resource.len() > 2 {
        match &resource[..3] {
            "arn" => convert_arn_to_dot_syntax(resource),
            // templated_string if &templated_string[..2] == "${" => {
            templated_string => {
                // TODO: maybe this should be done in attributes.rs extract_attributetype()
                let without_brackets = if &resource[..2] == "${" {
                    &resource[2..&resource.len()-1]
                } else {
                    resource
                };
                
                let attribute_suffix_removed = without_brackets.replace(".id", "").replace(".arn", "");
                
                Some(attribute_suffix_removed)
            },
            // others => Some(others.replace(".id", "").replace(".arn", "")),
        }
    } else {
        // we have hit a '*' wildcard
        Some(resource.to_string())
    }
}

fn extract_query_result(resource: &TFQueryResult) -> Vec<String> {
    match resource {
        Scalar(attribute) => attribute.extract_attributetype(),
        List(attributes) => attributes.into_iter().flat_map(|attribute| attribute.extract_attributetype()).collect(),
        TFQueryResult::None => vec![String::from("MASSIVE ERROR!!!")],
    }
}

fn hack_lambda_attributes(key: &String, val: String) -> String {
    if key == "function_name" && !val.contains(".") {
        "aws_lambda_function.".to_owned() + &val
    } else {
        val
    }
}

pub fn parse(block: &TerraformBlockWithTwoIdentifiers, aws_relationship_specs: &HashMap<String, Relationship>) -> Vec<Relationship> {
    let mut relationships: Vec<Relationship> = vec![];

    match aws_relationship_specs.get(&block.first_identifier) {
        Some(Relationship { source, target, label: _ }) => {
            let source_tf_result = jmespath_query(&block, source.as_str());
            let target_tf_result = jmespath_query(&block, target.as_str());

            let extracted_source = extract_query_result(&source_tf_result);
            let extracted_target = extract_query_result(&target_tf_result);
            
            let cleaned_source = clean_resource_identifier(&extracted_source[0]);
            let cleaned_targets: Vec<String> = extracted_target.into_iter().flat_map(|target| clean_resource_identifier(&target)).collect();
            
            if let Some(source_val) = cleaned_source {
                if cleaned_targets.len() > 0 {
                    let lambda_hack_source = hack_lambda_attributes(source, source_val);
                    let mut new_relationships: Vec<Relationship> = cleaned_targets.into_iter()
                        .map(|target_id| hack_lambda_attributes(target, target_id))
                        .map(|target_id| Relationship { source: lambda_hack_source.clone(), target: target_id, label: String::from("") })
                        .collect();

                    relationships.append(&mut new_relationships);
                }
            }
        },
        None => print!(""),
    }
    
    relationships.dedup_by(|a, b| a.target == b.target);
    relationships
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::template_string::TemplateString::Variable;
    use crate::structs::attributes::AttributeType::{ Boolean, Array, Num, Str, TemplatedString };

    #[test]
    fn extract_scalar_identifier_test() {
        let resource = Scalar(TemplatedString(Variable(String::from("aws_iam_role.discovery_vpc-flow-log-role.id"))));
        let expected = vec![
            String::from("aws_iam_role.discovery_vpc-flow-log-role.id")
        ];
        let result = extract_query_result(&resource);
        assert_eq!(result, expected)
    }

    #[test]
    fn extract_list_identifier_test() {
        let resource = List(vec![
            Array(vec![
                Str(String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}"))
            ]),
            Array(vec![
                Str(String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}:*"))
            ]), 
            Array(vec![
                Str(String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}:*"))
            ])
        ]);
        let expected = vec![
            String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}"),
            String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}:*"),
            String::from("arn:aws:logs:us-east-1:309983114184:log-group:${aws_cloudwatch_log_group.discovery_vpc-flow-log-group.name}:*"),
        ];
        let result = extract_query_result(&resource);
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_dynamo() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:dynamodb:us-east-1:309983114184:table/discovery_provider-consistency"));
        let expected = Some(String::from("aws_dynamodb_table.discovery_provider-consistency"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_s3_ending_in_wildcard() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:s3:::acp-platform-s-discovery-sandbox1/env/*"));
        let expected = Some(String::from("aws_s3_bucket.acp-platform-s-discovery-sandbox1/env"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_s3() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:s3:::acp-platform-s-discovery-sandbox1"));
        let expected = Some(String::from("aws_s3_bucket.acp-platform-s-discovery-sandbox1"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_sns() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:sns:us-east-1:309983114184:discovery_provider-inconsistency-topic"));
        let expected = Some(String::from("aws_sns_topic.discovery_provider-inconsistency-topic"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_lambda() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:lambda:us-east-1:309983114184:function:discovery_provider-consistency-scheduler"));
        let expected = Some(String::from("aws_lambda_function.discovery_provider-consistency-scheduler"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_sqs() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:sqs:us-east-1:309983114184:sre_auto-remediation-queue"));
        let expected = Some(String::from("aws_sqs_queue.sre_auto-remediation-queue"));
        assert_eq!(result, expected)
    }

    #[test]
    fn arn_conversion_logs() {
        let result = convert_arn_to_dot_syntax(&String::from("arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_remediate-missing-resources*"));
        let expected = None;
        assert_eq!(result, expected)
    }
}
