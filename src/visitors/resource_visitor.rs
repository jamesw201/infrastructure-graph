use std::cell::RefCell;
use std::collections::HashMap;

use crate::terraform::{
    TerraformBlock,
    TerraformBlock::{
        NoIdentifiers,
        WithOneIdentifier,
        WithTwoIdentifiers,
    },
    TerraformBlockWithNoIdentifiers,
    TerraformBlockWithOneIdentifier,
    TerraformBlockWithTwoIdentifiers,
    Attribute,
    AttributeType::{Str, Block}
};

use crate::visitors::visitor::Visitor;
use crate::visitors::json_visitor::JsonVisitor;
use crate::visitors::relationship_visitor::{RelationshipVisitor, Relationship};
use crate::relationship_finders::relationship_finder::RelationshipFinder;

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

pub fn dispatch(resources: &Vec<TerraformBlock>, aws_relationship_specs: HashMap<String, Relationship>) -> String {
    let mut vec = Vec::new();

    let json_visitor = JsonVisitor{relationships: RefCell::new(vec)};

    let visitor = RelationshipVisitor{ downstream_visitor: json_visitor, aws_relationship_specs };

    let json_resources: Vec<String> = resources.into_iter().map(|resource| visitor.visit_tfblock(resource)).collect();
    let json_resources_joined = json_resources.join(",");
    let relationships = visitor.output_relationships();

    format!("{{\"resources\":[{}],\"relationships\":{}}}", json_resources_joined, relationships)
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
