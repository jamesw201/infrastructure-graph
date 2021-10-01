use serde::{Deserialize, Serialize};
use crate::structs::terraform_block::TerraformBlock;
use crate::structs::template_string::{ TemplateString };

use crate::structs::traits::query::Queryable;
use crate::relationship_finders::tf_block_query::tf_block_query::{JmespathExpression, QueryPart};
// use QueryPart::{ List, Scalar};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum AttributeType {
    Str(String),
    TemplatedString(TemplateString),
    Boolean(bool),
    Num(f64),
    Array(Vec<AttributeType>),
    Block(Vec<Attribute>),
    TFBlock(TerraformBlock),
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeType
}

impl AttributeType {
    pub fn extract_attributetype(&self) -> Vec<String> {
        match self {
            Self::Str(value) => vec![value.to_string()],
            Self::Boolean(value) => vec![value.to_string()],
            Self::Num(value) => vec![value.to_string()],
            Self::TemplatedString(TemplateString::Variable(value)) => vec![value.to_string()],
            Self::Array(values) => values.into_iter().flat_map(|attr| attr.extract_attributetype()).collect(),
            _ => vec![String::from("MASSIVE ERROR!!!")],
        }
    }
}

impl Queryable for Attribute {
    fn query(&self, expression: &JmespathExpression) -> Vec<AttributeType> {
        // if: we're at the end of the expression then filter on it
        // else: find the next element of the search and drop the head of the expression parts
        if let Some((head, tail)) = expression.path_parts.split_first() {
            match head {
                QueryPart::Scalar(value) | QueryPart::List(value) => {
                    if &self.key == value {
                        if expression.path_parts.len() == 1 {
                            vec![self.value.clone()]
                        } else {
                            let new_expression = JmespathExpression { path_parts: tail.to_vec() };
                            self.value.query(&new_expression)
                        }
                    } else {
                        vec![]
                    }
                },
                _ => vec![]
            }
        } else {
            vec![]
        }
    }
}

impl Queryable for AttributeType {
    fn query(&self, expression: &JmespathExpression) -> Vec<AttributeType> {
        let as_strings = match self {
            Self::TFBlock(value) => value.query(expression),
            Self::Block(value) => {
                let attribute_types = value.into_iter().flat_map(|item| item.query(&expression)).collect();
                // println!("block attribute_types: {:?}", attribute_types);
                attribute_types
            },
            Self::Array(value) => {
                let attribute_types = value.into_iter().flat_map(|item| item.query(&expression)).collect();
                // println!("array attribute_types: {:?}", attribute_types);
                attribute_types
            },
            _ => vec![self.clone()]
        };
        // println!("result: {:?}", as_strings);
        as_strings
    }
}
