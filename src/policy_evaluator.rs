use crate::structs::terraform_block::{
    TerraformBlock,
};
use crate::structs::attributes::AttributeType;
use AttributeType::{ Block, Str, Num };
use crate::structs::policies::{ Policies, Policy, Filter };
use crate::relationship_finders::tf_block_query::tf_block_query::{ jmespath_query, TFQueryResult };

use TFQueryResult::{ List, Scalar };
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use itertools::Itertools;


#[derive(Debug, PartialEq)]
pub struct FilterResult {
    filter: Filter, // TODO: this should be a Vector of Filters
    result: bool,
}

impl FilterResult {
    pub fn new(filter: Filter, result: bool) -> FilterResult {
        FilterResult { filter, result }
    }
}

#[derive(Debug, PartialEq)]
pub struct PolicyResult {
    filter_results: Vec<FilterResult>, // TODO: this should be a Vector of Filters
    resource_id: String,
    result: bool,
}

impl PolicyResult {
    pub fn new(filter_results: Vec<FilterResult>, resource_id: String, result: bool) -> PolicyResult {
        PolicyResult { filter_results, resource_id, result }
    }
}

fn extract_policy_targets(policies: &Policies) -> Vec<String> {
    let result: Vec<String> = policies.policies.iter().map(|policy| {
        policy.resource.clone()
    }).collect();

    result.into_iter().unique().collect_vec()
}

fn evaluate_filter(filter: &Filter, attribute_type: AttributeType) -> FilterResult {
    let result = match attribute_type {
        Block(attributes) => {
            match &attributes[0].value {
                Str(val) => val == &filter.value,
                Num(val) => val == &filter.value.parse::<f64>().unwrap(),
                // TODO: accommodate all variants
                _ => false,
            }
        },
        // TODO: accommodate all variants
        _ => false,
    };

    FilterResult::new(filter.clone(), result)
}

fn evaluate_policy(policy: &Policy, resource: &TerraformBlock) -> PolicyResult {
    let filter_results: Vec<FilterResult> = policy.filters.iter().map(|filter| {
        let query_result = match resource {
            TerraformBlock::WithTwoIdentifiers(tf_block) => jmespath_query(tf_block, &filter.key.as_str()),
            _ => TFQueryResult::None,
        };

        match query_result {
            List(attribute_types) => FilterResult::new(filter.clone(), false),
            Scalar(attribute_type) => evaluate_filter(filter, attribute_type),
            TFQueryResult::None => FilterResult::new(filter.clone(), false),
        }
    }).collect();
    println!("filter_results: {:?}", filter_results);

    let combined_result = filter_results.iter().fold(true, |acc, x| acc && x.result);

    PolicyResult::new(filter_results, resource.get_id(), combined_result)
}

// TODO: Return a HashMap<Policy, Vec<PolicyResult>>
// fn query_resources(cache: HashMap<&str, Vec<&TerraformBlock>>, policies: Policies) -> HashMap<Policy, Vec<PolicyResult>> {
fn query_resources(cache: HashMap<&str, Vec<&TerraformBlock>>, policies: Policies) -> Vec<Vec<PolicyResult>> {
    let mut results_map: HashMap<&Policy, Vec<PolicyResult>> = HashMap::new();

    policies.policies.iter().map(|policy| {
        let cache_entry = cache.get(policy.resource.as_str());

        if let Some(resources) = cache_entry {
            Some(
                resources.iter().map(|resource| evaluate_policy(policy, resource)).collect()
                // results_map.insert(policy, resource_query_result);
            )
        } else {
            None
        }
    }).flatten().collect()
}

pub fn evaluate(policies: Policies, resources: Vec<TerraformBlock>) -> Vec<PolicyResult> {
    let mut cache: HashMap<&str, Vec<&TerraformBlock>> = HashMap::new();

    let resource_targets = extract_policy_targets(&policies);
    // println!("resource_targets: {:?}", &resource_targets);

    for target_resource in &resource_targets {
        let filtered_resources: &Vec<&TerraformBlock> = &resources.iter().filter(|&resource| {
            match resource {
                TerraformBlock::NoIdentifiers(_) => false,
                TerraformBlock::WithOneIdentifier(tf_block) => &tf_block.first_identifier == target_resource,
                TerraformBlock::WithTwoIdentifiers(tf_block) => &tf_block.first_identifier == target_resource,
            }
        }).collect();
        // println!("filtered_resources: {:?}", filtered_resources);

        cache.insert(target_resource, filtered_resources.clone());
    }
    // println!("cache: {:?}", &cache);

    let policy_results = query_resources(cache, policies);
    
    println!("policy_results: {:?}", policy_results);
    // TODO: 
    // [√] loop through Policies
    // [√] find the corresponding resources
    // [√] build a map of evaluation results
    // [ ] flatten the map and return

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::terraform_block::{
        TerraformBlock,
        TerraformBlockWithTwoIdentifiers,
    };
    use crate::structs::attributes::{
        Attribute,
        AttributeType,
    };
    use crate::structs::json::JsonValue;
    use crate::structs::policies::{ Policies, Policy, Filter };

    fn setup_policies() -> Policies {
        Policies {
            policies: vec![
                Policy {
                    name: String::from("example-do-policy-name-check"),
                    description: String::from("balbabal"),
                    resource: String::from("aws_iam_role_policy"),
                    filters: vec![
                        Filter {
                            key: String::from("policy.maxReceiveCount"),
                            op: String::from("eq"),
                            value: String::from("2.0"),
                        },
                    ],
                },
                Policy {
                    name: String::from("example-do-ec2-id-check"),
                    description: String::from("balbabal"),
                    resource: String::from("aws_ec2_instance"),
                    filters: vec![
                        Filter {
                            key: String::from("id"),
                            op: String::from("eq"),
                            value: String::from("id-that-we-are-looking-for"),
                        },
                    ],
                },
                Policy {
                    name: String::from("policy-id-check"),
                    description: String::from("balbabal"),
                    resource: String::from("aws_iam_role_policy"),
                    filters: vec![
                        Filter {
                            key: String::from("visibility_timeout_seconds"),
                            op: String::from("eq"),
                            value: String::from("30.0"),
                        },
                    ],
                },
            ]
        }
    }

    fn setup_resources() -> Vec<TerraformBlock> {
        vec![
            TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type: String::from("resource"),
                    first_identifier: String::from("aws_iam_role_policy"),
                    second_identifier: String::from("my-iam-policy"),
                    attributes: vec![
                        Attribute {
                            key: String::from("visibility_timeout_seconds"), 
                            value: AttributeType::Num(30.0)
                        },
                        Attribute {
                            key: String::from("policy"),
                            value: AttributeType::Json(JsonValue::Object(vec![
                                (String::from("deadLetterTargetArn"), JsonValue::Str(String::from("${aws_sqs_queue.discovery_collector-deadletter-queue.arn}"))),
                                (String::from("maxReceiveCount"), JsonValue::Num(2.0))]))
                        },
                    ]
                }
            ),
            TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type: String::from("resource"),
                    first_identifier: String::from("aws_iam_role_policy"),
                    second_identifier: String::from("second-iam-policy"),
                    attributes: vec![]
                }
            ),
            TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type: String::from("resource"),
                    first_identifier: String::from("aws_ec2_instance"),
                    second_identifier: String::from("my_ec2_instance"),
                    attributes: vec![]
                }
            ),
        ]
    }

    #[test]
    fn evaluate_filter_test() {
        let policies = setup_policies();

        let filter = Filter {
            key: String::from("policy.maxReceiveCount"),
            op: String::from("eq"),
            value: String::from("2.0"),
        };
        let filter_result = FilterResult::new(filter, true);

        let attribute_input = AttributeType::Block(
            vec![Attribute{ key: String::from("policy.maxReceiveCount"), value: AttributeType::Num(2.0) }]
        );
        let result = evaluate_filter(&policies.policies[0].filters[0], attribute_input);
        assert_eq!(result, filter_result)
    }

    #[test]
    fn evaluate_policy_test() {
        let resources = setup_resources();
        let policies = setup_policies();

        let result = evaluate_policy(&policies.policies[0], &resources[0]);
        let filter_result = FilterResult {
            filter: Filter {
                key: String::from("policy.maxReceiveCount"),
                op: String::from("eq"),
                value: String::from("2.0"),
            },
            result: true,
        };
        let expected = PolicyResult{
            filter_results: vec![filter_result],
            resource_id: resources[0].get_id(),
            result: true,
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn unique_policy_resources_test() {
        let policies = setup_policies();

        let result = extract_policy_targets(&policies);
        assert_eq!(result, vec![String::from("aws_iam_role_policy"), String::from("aws_ec2_instance")])
    }

    #[test]
    fn unique_resources_test() {
        let resources = setup_resources();
        let policies = setup_policies();

        let result = evaluate(policies, resources);
        assert_eq!(1, 2)
    }
}
