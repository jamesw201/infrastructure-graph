use crate::structs::terraform_block::TerraformBlock;
use crate::structs::template_string::{ TemplateString };
use crate::structs::json::JsonValue;


#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeType
}