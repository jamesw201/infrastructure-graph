use serde::{Deserialize, Serialize};

use crate::structs::attributes::{ Attribute, AttributeType };
use crate::structs::traits::query::Queryable;
use crate::relationship_finders::tf_block_query::tf_block_query::{JmespathExpression, PathPart};
use PathPart::{ List, Scalar};

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

impl Queryable for TerraformBlock {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType> {
        match self {
            Self::NoIdentifiers(TerraformBlockWithNoIdentifiers) => None,
            Self::WithOneIdentifier(TerraformBlockWithOneIdentifier) => None,
            Self::WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers) => TerraformBlockWithTwoIdentifiers.query(expression)
        }
    }
}

impl Queryable for TerraformBlockWithTwoIdentifiers {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType> {

        let found = self.attributes.iter().find(|attr| {
            match &expression.path_parts[0] {
                Scalar(path) => attr.key == path.to_string(),
                List(path) => attr.key == path.to_string(),
            }
        });

        match found {
            Some(thing) => {
                if expression.path_parts.len() == 1 {
                    Some(thing.value.clone())
                } else {
                    println!("path_parts.len() == {}", expression.path_parts.len());
                    let new_expression = JmespathExpression { path_parts: expression.path_parts[1..].to_vec() };
                    thing.value.query(new_expression)
                }
            },
            None => None
        }
    }
}
