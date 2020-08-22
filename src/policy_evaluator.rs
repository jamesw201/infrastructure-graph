use crate::structs::terraform_block::{
    TerraformBlock,
};
use crate::structs::attributes::AttributeType;
use AttributeType::{ Block, Str, Num };
use crate::structs::policies::{ Policies, Policy, Filter };
use crate::relationship_finders::tf_block_query::tf_block_query::{ jmespath_query, TFQueryResult };

use TFQueryResult::{ List, Scalar };
use std::collections::HashMap;
use itertools::Itertools;


#[derive(Debug, PartialEq, Clone)]
pub struct FilterResult {
    filter: Filter, // TODO: this should be a Vector of Filters
    result: bool,
}

impl FilterResult {
    pub fn new(filter: Filter, result: bool) -> FilterResult {
        FilterResult { filter, result }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PolicyResult {
    filters: Vec<FilterResult>,
    policy_id: String,
    policy_result: bool,
}

impl PolicyResult {
    pub fn new(filters: Vec<FilterResult>, policy_id: String, policy_result: bool) -> PolicyResult {
        PolicyResult { filters, policy_id, policy_result }
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
        Str(val) => val == filter.value,
        Num(val) => val == filter.value.parse::<f64>().unwrap(),
        // TODO: accommodate all variants
        _ => false,
    };

    FilterResult::new(filter.clone(), result)
}

fn evaluate_policy(policy: &Policy, resource: &TerraformBlock) -> PolicyResult {
    let filters: Vec<FilterResult> = policy.filters.iter().map(|filter| {
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
    println!("filters: {:?}", filters);

    let combined_result = filters.iter().fold(true, |acc, x| acc && x.result);

    PolicyResult::new(filters, policy.name.to_string(), combined_result)
}

// TODO: Return a HashMap<Policy, Vec<PolicyResult>>
// fn query_resources(cache: HashMap<&str, Vec<&TerraformBlock>>, policies: Policies) -> HashMap<Policy, Vec<PolicyResult>> {
fn query_resources<'a>(cache: HashMap<&str, Vec<&TerraformBlock>>, policies: Policies) -> HashMap<String, Vec<PolicyResult>> {
    let mut results_map: HashMap<String, Vec<PolicyResult>> = HashMap::new();

    for policy in policies.policies {
        let cache_entry = cache.get(policy.resource.as_str());

        if let Some(resources) = cache_entry {
            for &resource in resources {
                let policy_results = evaluate_policy(&policy, &resource);

                if policy_results.policy_result == false {
                    let existing_policy_results = results_map.get(&resource.get_id());

                    match existing_policy_results {
                        Some(results) => {
                            let mut bla = results.clone();
                            bla.push(policy_results);
                            println!("old vec: {:?}", existing_policy_results);
                            println!("new vec: {:?}", bla);
                            results_map.insert(resource.get_id(), bla.to_vec());
                        },
                        None => {
                            results_map.insert(resource.get_id(), vec![policy_results]);
                        },
                    }
                }
            }
        }
    };

    results_map
}

pub fn evaluate(policies: Policies, resources: &Vec<TerraformBlock>) -> HashMap<String, Vec<PolicyResult>> {
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
    
    println!("policies: {:?}", policy_results);

    policy_results
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
                        Filter::new("policy.maxReceiveCount", "eq", "2.0"),
                    ],
                },
                Policy {
                    name: String::from("example-do-ec2-id-check"),
                    description: String::from("balbabal"),
                    resource: String::from("aws_ec2_instance"),
                    filters: vec![
                        Filter::new("id", "eq", "id-that-we-are-looking-for2"),
                    ],
                },
                Policy {
                    name: String::from("policy-id-check"),
                    description: String::from("balbabal"),
                    resource: String::from("aws_iam_role_policy"),
                    filters: vec![
                        Filter::new("visibility_timeout_seconds", "eq", "3.1"),
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
                    attributes: vec![
                        Attribute {
                            key: String::from("visibility_timeout_seconds"), 
                            value: AttributeType::Num(30.0)
                        },
                    ]
                }
            ),
            TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type: String::from("resource"),
                    first_identifier: String::from("aws_ec2_instance"),
                    second_identifier: String::from("my_ec2_instance"),
                    attributes: vec![
                        Attribute {
                            key: String::from("id"),
                            value: AttributeType::Str(String::from("id-that-we-are-looking-for"))
                        }
                    ]
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
            filters: vec![filter_result],
            policy_id: policies.policies[0].name.to_string(),
            policy_result: true,
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
    fn evaluate_test() {
        let resources = setup_resources();
        let policies = setup_policies();
        let policies_clone = policies.clone();

        let result = evaluate(policies, &resources);
        let mut expected_map: HashMap<String, Vec<PolicyResult>> = HashMap::new();
        
        let f1 = Filter::new("id", "eq", "id-that-we-are-looking-for2");
        let f2 = Filter::new("visibility_timeout_seconds", "eq", "3.1");
        let f3 = Filter::new("policy.maxReceiveCount", "eq", "2.0");
        let filter_result1 = FilterResult::new(f1, false);
        let filter_result2= FilterResult::new(f2, false);
        let filter_result3 = FilterResult::new(f3, false);

        let policyResult1 = PolicyResult::new(vec![filter_result1.clone()], policies_clone.policies[0].name.to_string(), false);
        let policyResult2 = PolicyResult::new(vec![filter_result2.clone()], policies_clone.policies[0].name.to_string(), false);
        let policyResult2_clone = PolicyResult::new(vec![filter_result2.clone()], policies_clone.policies[0].name.to_string(), false);
        let policyResult3 = PolicyResult::new(vec![filter_result3.clone()], policies_clone.policies[0].name.to_string(), false);
        
        expected_map.insert(resources[2].get_id(), vec![policyResult1]);
        expected_map.insert(resources[0].get_id(), vec![policyResult2]);
        expected_map.insert(resources[1].get_id(), vec![policyResult2_clone, policyResult3]);

        assert_eq!(result.contains_key(&resources[0].get_id()), true);
        assert_eq!(result.contains_key(&resources[1].get_id()), true);
        assert_eq!(result.contains_key(&resources[2].get_id()), true);
    }
}
