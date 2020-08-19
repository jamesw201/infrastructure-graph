use serde::{Deserialize, Serialize};
use crate::structs::terraform_block::TerraformBlock;
use crate::structs::template_string::{ TemplateString };
use crate::structs::json::JsonValue;

use crate::structs::traits::query::Queryable;
use crate::relationship_finders::tf_block_query::tf_block_query::{JmespathExpression, PathPart};
use PathPart::{ List, Scalar};


#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum AttributeType {
    Str(String),
    TemplatedString(TemplateString),
    Boolean(bool),
    Num(f64),
    Array(Vec<AttributeType>),
    Block(Vec<Attribute>),
    TFBlock(TerraformBlock),
    Json(JsonValue),
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeType
}

impl Queryable for Attribute {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType> {
        println!("unhandled Attribute");
        None
    }
}

impl Queryable for AttributeType {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType> {
        match self {
            Self::TFBlock(value) => value.query(expression),
            Self::Json(value) => value.query(expression),
            _ => None   // any other value would not make sense if the JmespathExpression hasn't reached its leaf node here
        }
    }
}
