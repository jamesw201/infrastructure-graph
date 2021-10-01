use std::cell::RefCell;
use std::collections::HashMap;

use crate::structs::terraform_block::{
    TerraformBlock,
    TerraformBlock::{ NoIdentifiers, WithOneIdentifier, WithTwoIdentifiers },
    TerraformBlockWithTwoIdentifiers,
};

use crate::visitors::visitor::Visitor;
use crate::visitors::json_visitor::JsonVisitor;
use crate::structs::policies::Policies;
// use crate::visitors::relationship_visitor::{RelationshipVisitor};s
// use crate::relationship_finders::relationship_finder::RelationshipFinder;
use crate::visitors::relationship_finder::{ Relationship };
use crate::policy_evaluator;
use crate::visitors::relationship_finder;
use crate::occlusion_engine;


pub fn dispatch(resources: &Vec<TerraformBlock>, aws_relationship_specs: HashMap<String, Relationship>, 
    policies: Policies, occlusion_specs: HashMap<String, Vec<String>>) -> String {
    // let mut vec = Vec::new();

    // let json_visitor = JsonVisitor{relationships: RefCell::new(vec)};
    // let visitor = JsonVisitor{relationships: RefCell::new(vec)};
    let visitor = JsonVisitor{};

    // let visitor = RelationshipVisitor{ downstream_visitor: json_visitor, aws_relationship_specs };

    let json_resources: Vec<String> = resources.into_iter().map(|resource| visitor.visit_tfblock(resource)).collect();
    let json_resources_joined = json_resources.join(",");
    // let relationships = visitor.output_relationships();
    
    println!("resources length: {}", &resources.len());
    let relationships: Vec<Relationship> = resources.into_iter().flat_map(|resource| {
        if let WithTwoIdentifiers(block) = resource {
            Some(relationship_finder::parse(block, &aws_relationship_specs))
        } else {
            None
        }
    }).flatten().collect();
    // println!("{} relationships: {:?}", relationships.len(), relationships);

    // TODO: create Graph(resources, relationships)
    let infra_graph: InfrastructureGraph = InfrastructureGraph::new(resources, relationships);

    // TODO: policy_evaluator::evaluate(policies, infra_graph);
    let policy_results: PolicyGraph = policy_evaluator::evaluate(policies, resources);
    
    let serialized = serde_json::to_string(&policy_results).unwrap();

    let joined_relationships = relationships.iter().map(|rel| format!("{}", rel)).collect::<Vec<String>>().join(",");
    // format!("[{}]", joined_relationships)

    // TODO: 
    // [ ] execute occlusions for the given provider
    let occluded_graph: OccludedPolicyGraph = occlusion_engine::occlude(policy_graph);

    // TODO: move most of these calls out of here and compose them in the main file

    format!(r#"{{"resources":[{}],"relationships":[{}],"policy_results":{}}}"#, json_resources_joined, joined_relationships, serialized)
    // format!(r#"{{"resources":[{}],"policy_results":{}}}"#, json_resources_joined, serialized)
    // format!(r#"{{"relationships":{:?}}}"#, relationships)
    // format!(r#"{{"blarp":"blarp"}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn parse_test() {
    //     let resource1 = WithOneIdentifier(
    //         TerraformBlockWithOneIdentifier {
    //             block_type: String::from("resource"),
    //             first_identifier: String::from("thing1"),
    //             attributes: vec![]
    //         }
    //     );
    //     let resource2 = WithTwoIdentifiers(
    //         TerraformBlockWithTwoIdentifiers {
    //             block_type: String::from("data"),
    //             first_identifier: String::from("terraform_remote_state"),
    //             second_identifier: String::from("acp-platform-s-discovery-sandbox1_remote_state"),
    //             attributes: vec![
    //                 Attribute {
    //                     key: String::from("config"),
    //                     value: Block(vec![
    //                         Attribute {
    //                             key: String::from("bucket"),
    //                             value: Str(String::from("acp-platform-s-discovery-sandbox1"))
    //                         },
    //                         Attribute {
    //                             key: String::from("key"),
    //                             value: Str(String::from("infrastructure/terraform.tfstate"))
    //                         },
    //                         Attribute {
    //                             key: String::from("region"),
    //                             value: Str(String::from("us-east-1"))
    //                         },
    //                     ]) 
    //                 }, 
    //                 Attribute {
    //                     key: String::from("backend"),
    //                     value: Str(String::from("s3"))
    //                 }
    //             ]
    //         }
    //     );
    //     dispatch(&vec![resource1, resource2]);
    //     assert_eq!(1,2)
    // }
}
