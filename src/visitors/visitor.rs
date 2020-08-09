use crate::structs::terraform_block::{
    TerraformBlock,
};

use crate::structs::attributes::{ Attribute, AttributeType };
use crate::structs::template_string::{ TemplateString };
use crate::structs::json::JsonValue;


pub trait Visitor<T> {
    fn visit_str(&self, value: &String) -> T;
    fn visit_template_string(&self, value: &TemplateString) -> T;
    fn visit_boolean(&self, value: &bool) -> T;
    fn visit_num(&self, value: &f64) -> T;
    fn visit_block(&self, value: &Vec<Attribute>) -> T;
    fn visit_array(&self, value: &Vec<AttributeType>) -> T;
    fn visit_tfblock(&self, value: &TerraformBlock) -> T;
    fn visit_attribute(&self, value: &Attribute) -> T;
    fn visit_json(&self, value: &JsonValue) -> T;
    fn visit_json_array(&self, value: &Vec<JsonValue>) -> T;
    fn visit_json_object(&self, value: &Vec<(String, JsonValue)>) -> T;
}

pub trait PolicyEvaluator {
    fn evalutate_policy() -> String;
}
