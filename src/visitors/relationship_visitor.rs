use std::cell::RefCell;
use std::fmt;
use std::marker::Copy;
use serde::{Deserialize};
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
    AttributeType,
    AttributeType::{Str, TemplatedString, Boolean, Num, Array, Block, TFBlock, Json},
    TemplateString,
    TemplateString::{Variable, BuiltInFunction},
};
use crate::json::{
    JsonValue
};
use crate::visitors::visitor::{ Visitor };
use crate::visitors::json_visitor::JsonVisitor;
use crate::relationship_finders::relationship_finder::{RelationshipFinder};

#[derive(Deserialize, Debug)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub label: String,
}

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"in\":\"{}\",\"out\":\"{}\",\"label\":\"{}\"}}", self.source, self.target, self.label)
    }
}

pub struct RelationshipVisitor {
    pub downstream_visitor: JsonVisitor,
    pub aws_relationship_specs: HashMap<String, Relationship>,
}

impl RelationshipVisitor {
    pub fn extract_template_string(attribute: Option<&Attribute>) -> String {
        match attribute {
            Some(Attribute { key, value: TemplatedString( template_string ) }) => {
                match template_string {
                    Variable(v_string) => {
                        v_string.to_string()
                    },
                    _ => String::from("")
                }
            },
            Some(_) => String::from(""),
            None => String::from("")
        }
    }
}

impl RelationshipFinder for RelationshipVisitor {
    fn add_relationship(&self, relationship: Relationship) {
        self.downstream_visitor.relationships.borrow_mut().push(relationship);
    }

    fn output_relationships(&self) -> String {
        self.downstream_visitor.output_relationships()
    }
}

impl Visitor<String> for RelationshipVisitor {
    fn visit_str(&self, value: &String) -> String {
        self.downstream_visitor.visit_str(value)
    }

    fn visit_template_string(&self, value: &TemplateString) -> String {
        self.downstream_visitor.visit_template_string(value)
    }

    fn visit_boolean(&self, value: &bool) -> String {
        self.downstream_visitor.visit_boolean(value)
    }

    fn visit_num(&self, value: &f64) -> String {
        self.downstream_visitor.visit_num(value)
    }

    fn visit_block(&self, value: &Vec<Attribute>) -> String {
        self.downstream_visitor.visit_block(value)
    }

    fn visit_array(&self, value: &Vec<AttributeType>) -> String {
        self.downstream_visitor.visit_array(value)
    }

    fn visit_tfblock(&self, value: &TerraformBlock) -> String {
        let bla: String = match value {
            NoIdentifiers(
                TerraformBlockWithNoIdentifiers {
                    block_type,
                    attributes
                }
            ) => {
                let attributes_json: Vec<String> = attributes.into_iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");

                format!("{{\"type\":\"{}\",\"body\":{{{}}}}}", block_type, attributes_joined)
            },
            WithOneIdentifier(
                TerraformBlockWithOneIdentifier {
                    block_type,
                    first_identifier,
                    attributes
                }
            ) => {
                let attributes_json: Vec<String> = attributes.into_iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");

                format!("{{\"type\":\"{}\",\"name\":\"{}\",\"body\":{{{}}}}}", block_type, first_identifier, attributes_joined)
            },
            WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type,
                    first_identifier,
                    second_identifier,
                    attributes
                }
            ) => {
                match self.aws_relationship_specs.get(first_identifier) {
                    Some(Relationship { source, target, label }) => {
                        // search through attributes for appropriate fields
                        let rel = Relationship { source: source.to_string(), target: target.to_string(), label: label.to_string() };

                        let source = attributes.into_iter().find(|&attr| attr.key == source.to_string());
                        let target = attributes.into_iter().find(|&attr| attr.key == target.to_string());

                        let source_t_string = Self::extract_template_string(source);
                        let target_t_string = Self::extract_template_string(target);

                        if !source_t_string.is_empty() && !target_t_string.is_empty() {
                            let relationship = Relationship { source: source_t_string, target: target_t_string, label: String::from("blarp") };
                            self.downstream_visitor.add_relationship(relationship)
                        }
                    },
                    None => print!(""),
                };

                let attributes_json: Vec<String> = attributes.into_iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");
                format!("{{\"type\":\"{}\",\"name\":\"{}\",\"body\":{{{}}}}}", first_identifier, second_identifier, attributes_joined)
            },
        };

        // [ ] determine if this block represents an IAM Policy or other relevant resource
        // [ ] somehow determine if attribute values are TemplateStrings
        // [ ] if found build a relationship between the reference in their value and the name of this block
        // [ ] somehow add the relationship to a the downstream_visitor's context object
        self.downstream_visitor.visit_tfblock(value)
    }

    fn visit_attribute(&self, value: &Attribute) -> String {
        self.downstream_visitor.visit_attribute(value)
    }

    fn visit_json(&self, value: &JsonValue) -> String {
        self.downstream_visitor.visit_json(value)
    }

    fn visit_json_array(&self, value: &Vec<JsonValue>) -> String {
        self.downstream_visitor.visit_json_array(value)
    }

    fn visit_json_object(&self, value: &Vec<(String, JsonValue)>) -> String {
        self.downstream_visitor.visit_json_object(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockJsonVisitor {
        pub relationships: Vec<String>,
    }

    impl Visitor<String> for MockJsonVisitor {
        fn visit_str(&self, value: &String) -> String { String::from("") }
        fn visit_template_string(&self, value: &TemplateString) -> String { String::from("") }
        fn visit_boolean(&self, value: &bool) -> String { String::from("") }
        fn visit_num(&self, value: &f64) -> String { String::from("") }
        fn visit_block(&self, value: &Vec<Attribute>) -> String { String::from("") }
        fn visit_array(&self, value: &Vec<AttributeType>) -> String { String::from("") }
        fn visit_tfblock(&self, value: &TerraformBlock) -> String { String::from("") }
        fn visit_attribute(&self, value: &Attribute) -> String { String::from("") }
        fn visit_json(&self, value: &JsonValue) -> String { String::from("") }
        fn visit_json_array(&self, value: &Vec<JsonValue>) -> String { String::from("") }
        fn visit_json_object(&self, value: &Vec<(String, JsonValue)>) -> String { String::from("") }
    }

    #[test]
    fn relationship_visitor_test() {
        let resource1 = WithOneIdentifier(
            TerraformBlockWithOneIdentifier {
                block_type: String::from("resource"),
                first_identifier: String::from("thing1"),
                attributes: vec![
                    Attribute {
                        key: String::from("backend"),
                        value: Str(String::from("s3"))
                    },
                    Attribute {
                        key: String::from("bookend"),
                        value: Boolean(true)
                    }
                ]
            }
        );
        let expected = String::from(r#"{"type":"resource","name":"thing1","body":{"backend":"s3","bookend":"true"}}"#);
        
        let mut vec = Vec::new();
        let mut h_map = HashMap::new();
        let visitor = RelationshipVisitor{
            downstream_visitor: JsonVisitor{relationships: RefCell::new(vec)},
            aws_relationship_specs: h_map,
        };  
        let result = visitor.visit_tfblock(&resource1);
        assert_eq!(result, expected)
    }
}