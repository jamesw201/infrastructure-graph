use serde::{Deserialize, Serialize};

use crate::structs::attributes::{ Attribute, AttributeType };
use crate::structs::traits::query::Queryable;
use crate::structs::traits::builder::Builder;

use crate::relationship_finders::tf_block_query::tf_block_query::{JmespathExpression, QueryPart};
// use PathPart::{ List, Scalar};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum TerraformBlock {
    NoIdentifiers(TerraformBlockWithNoIdentifiers),
    WithOneIdentifier(TerraformBlockWithOneIdentifier),
    WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TerraformBlockWithNoIdentifiers {
    pub block_type: String,
    pub attributes: Vec<Attribute>
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TerraformBlockWithOneIdentifier {
    pub block_type: String,
    pub first_identifier: String,
    pub attributes: Vec<Attribute>
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TerraformBlockWithTwoIdentifiers {
    pub block_type: String,
    pub first_identifier: String,
    pub second_identifier: String,
    pub attributes: Vec<Attribute>
}

impl TerraformBlock {
    // create a get_id function to return IDs
    pub fn get_id(&self) -> String {
        match self {
            Self::NoIdentifiers(resource) => resource.block_type.to_string(),
            Self::WithOneIdentifier(resource) => resource.block_type.to_string() + "_" + &resource.first_identifier,
            Self::WithTwoIdentifiers(resource) => resource.first_identifier.to_string() + "_" + &resource.second_identifier,
        }
    }
}

impl TerraformBlockWithTwoIdentifiers {
    pub fn new(block_type: &str, first_identifier: &str, second_identifier: &str, attributes: Vec<Attribute>) -> TerraformBlockWithTwoIdentifiers {
        TerraformBlockWithTwoIdentifiers { block_type: block_type.to_owned(), first_identifier: first_identifier.to_owned(),
            second_identifier: second_identifier.to_owned(), attributes }
    }
}

impl Queryable for TerraformBlock {
    fn query(&self, expression: &JmespathExpression) -> Vec<AttributeType> {
        match self {
            Self::NoIdentifiers(TerraformBlockWithNoIdentifiers) => vec![],
            Self::WithOneIdentifier(TerraformBlockWithOneIdentifier) => vec![],
            Self::WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers) => TerraformBlockWithTwoIdentifiers.query(expression)
        }
    }
}

impl Queryable for TerraformBlockWithTwoIdentifiers {
    fn query(&self, expression: &JmespathExpression) -> Vec<AttributeType> {

        if let Some((head, tail)) = expression.path_parts.split_first() {

            let found = self.attributes.iter().find(|attr| {
                match head {
                    QueryPart::Scalar(path) => attr.key == path.to_string(),
                    QueryPart::List(path) => attr.key == path.to_string(),
                }
            });

            match found {
                Some(thing) => {
                    if expression.path_parts.len() == 1 {
                        vec![thing.value.clone()]
                    } else {
                        let new_expression = JmespathExpression { path_parts: tail.to_vec() };
                        thing.value.query(&new_expression)
                    }
                },
                None => vec![]
            }
        } else {
            vec![]
        }
    }
}

// impl Builder<TerraformBlockWithTwoIdentifiers> for TerraformBlockWithTwoIdentifiers {
//     fn build_from_json_string(&self, input_string: &str) -> Result<TerraformBlockWithTwoIdentifiers, > {
//         // TODO:
//         // [ ] parse input_string as serde json
//         // [ ] create resource from json to struct
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use super::Builder;

    // #[test]
    // fn build_from_json_string_test() {
    //     let input_string = r#""#;
    //     let block = TerraformBlockWithTwoIdentifiers::new("resource", "aws_iam_role_policy", "discovery_scheduler_role_policy", vec![]);
    //     let result = block.build_from_json_string(input_string);
    //     assert_eq!(1, 2)
    // }
}