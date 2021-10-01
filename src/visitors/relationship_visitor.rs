use std::cell::RefCell;
use std::fmt;
use std::error::Error;
use std::marker::Copy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::relationship_finders::tf_block_query::tf_block_query::{ jmespath_query, TFQueryResult };

use crate::visitors::relationship_finder::{ Relationship };

use crate::structs::attributes::{ Attribute, AttributeType };
use AttributeType::{
    Array, Block, Boolean, Num, Str, TFBlock, TemplatedString,
};
use crate::structs::template_string::{ TemplateString };

use TemplateString::{ Variable, BuiltInFunction };

use crate::structs::terraform_block::{
    TerraformBlock,
    TerraformBlock::{
        NoIdentifiers,
        WithOneIdentifier,
        WithTwoIdentifiers,
    },
    TerraformBlockWithNoIdentifiers,
    TerraformBlockWithOneIdentifier,
    TerraformBlockWithTwoIdentifiers,
};

use crate::visitors::visitor::{ Visitor };
use crate::visitors::json_visitor::JsonVisitor;
use crate::relationship_finders::relationship_finder::{RelationshipFinder};

// #[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(untagged)]
// pub enum Relationship {
//     BasicRelationship { source: String, target: String, label: String },  
//     NestedRelationship { source: String, target: String, label: String },
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq)]
// pub struct Relationship {
//     pub source: String,
//     pub target: String,
//     pub label: String,
// }


pub struct RelationshipVisitor {
    pub downstream_visitor: JsonVisitor,
    pub aws_relationship_specs: HashMap<String, Relationship>,
}

impl RelationshipVisitor {
    pub fn extract_value(attribute: Option<&Attribute>) -> Option<String> {
        match attribute {
            Some(Attribute { key, value: TemplatedString( template_string ) }) => {
                match template_string {
                    Variable(v_string) => {
                        Some(v_string.to_string().replace(".id", "").replace(".arn", ""))
                        // Self::handle_resource(&v_string.to_string())
                    },
                    _ => Some(String::from(""))
                }
            },
            Some(Attribute { key, value: Str( str_val ) }) => {
                // Self::handle_resource(&str_val.to_string())
                Some(str_val.to_string().replace(".id", "").replace(".arn", ""))
            },
            // TODO: match on Str() for ARNs which were not templated
            Some(_) => Some(String::from("")),
            None => Some(String::from(""))
        }
    }

    /// ARNs are split into parts separated by a colon.
    /// From these six parts a terraform syntax name can be generated.
    pub fn convert_arn_to_dot_syntax(arn_resource: &String) -> Option<String> {
        let tokens: Vec<&str> = arn_resource.split(":").collect();

        let dot_syntax_resource = match tokens[2] {
            "dynamodb" => Some(format!("aws_dynamodb_table.{}", tokens[5].replace("table/", "").replace("/*", ""))),
            "sns" => Some(format!("aws_sns_topic.{}", tokens[5])),
            "s3" => {
                // let first_slash = tokens[5].find("/");

                // match first_slash {
                //     Some(position) => Some(format!("aws_s3_bucket.{}", &tokens[5][0..position])),
                //     None => Some(format!("aws_s3_bucket.{}", tokens[5])),
                // }
                if tokens[5].ends_with("/*") {
                    let without_wildcard = tokens[5].len() - 2;
                    Some(format!("aws_s3_bucket.{}", &tokens[5][0..without_wildcard]))
                } else {
                    Some(format!("aws_s3_bucket.{}", tokens[5]))
                }
            },
            "lambda" => Some(format!("aws_lambda_function.{}", tokens[6])),
            "sqs" => Some(format!("aws_sqs_queue.{}", tokens[5])),
            "kinesis" => {
                let first_slash = tokens[5].find("/");

                match first_slash {
                    Some(position) => Some(format!("aws_kinesis_stream.{}", &tokens[5][0..position])),
                    None => Some(format!("aws_kinesis_stream.{}", &tokens[5])),
                }
            },
            "logs" => None, // log relationships are not wanted on a diagram
            anything => {
                println!("unknown arn: {:?}, for resource: {}", anything, arn_resource);
                None
            },
        };

        dot_syntax_resource
    }

    fn handle_resource(resource: &String) -> Option<String> {
        if resource.len() > 2 {
            match &resource[..3] {
                "arn" => Self::convert_arn_to_dot_syntax(resource),
                // templated_string if &templated_string[..2] == "${" => {
                templated_string => {
                    let templating_chars_removed = &resource[2..&resource.len()-1];
                    
                    let attribute_suffix_removed = templating_chars_removed.replace(".id", "").replace(".arn", "");
                    
                    Some(attribute_suffix_removed)
                },
                // others => Some(others.replace(".id", "").replace(".arn", "")),
            }
        } else {
            // we have hit a '*' wildcard
            Some(resource.to_string())
        }
    }

    // build a TargetAndLabel from a policy statement object
    // fn handle_statements(&self, statements: &Vec<(String, JsonValue)>, collection_path: &str) -> Vec<TargetAndLabel> {    
    //     let action = statements.into_iter().find(|(key, value)| key.as_str() == "Action");
    //     // println!("action: {:?}", action);
    //     let effect = statements.into_iter().find(|(key, value)| key.as_str() == "Effect");
    //     // println!("effect: {:?}", effect);
    //     let resource = statements.into_iter().find(|(key, value)| key.as_str() == "Resource");
    //     // create a TargetAndLabel for each Resource found
    //     let targets: Vec<Option<String>> = match resource {
    //         Some((_, JsonValue::Array( items ))) => {
    //             items.into_iter().map(|item| {
    //                 match item {
    //                     JsonValue::Str(val) => {
    //                         Self::handle_resource(val)
    //                     },
    //                     anything => {
    //                         println!("unknown type: {:?}", anything);
    //                         None
    //                     },
    //                 }
    //             }).collect()
    //         },
    //         _ => vec![],
    //     };

    //     // loop through targets and add actions where a value is present
    //     let mut withDuplicates: Vec<TargetAndLabel> = targets.into_iter().map(|target| {
    //         let vals = match action {
    //             Some((key, JsonValue::Array(strings))) => self.downstream_visitor.visit_json_array(strings).replace("\"", "'"),
    //             _ => String::from(""),
    //         };
    //         let tget = match target {
    //             Some(tget_val) => tget_val,
    //             _ => String::from(""),
    //         };
    //         TargetAndLabel { collection_path: collection_path.to_string(), target: tget, label: vals}
    //     }).collect();

    //     // TODO:
    //     // [√] loop through targets and remove any duplicates
    //     // [ ] apply rules for merging Relationships?
    //     withDuplicates.dedup_by(|a, b| a.target == b.target);
    //     withDuplicates
    // }

    // pub fn extract_values(&self, attribute: Option<&Attribute>, collection_path: &str, target: &str, label: &str) -> Vec<TargetAndLabel> {
    //     let dot_split = collection_path.split(".").collect::<Vec<&str>>();

    //     match attribute {
    //         Some(Attribute { key, value: AttributeType::Json(JsonValue::Object( json_attributes )) }) => {

    //             let nested_attr = json_attributes.into_iter().find(|&(attr, _)| attr == dot_split[1]);

    //             match nested_attr {
    //                 Some((_, JsonValue::Array( elements ))) => { // mapping over statement objects to create
    //                     elements.iter().flat_map(|obj:&JsonValue| {
    //                         match obj {
    //                             JsonValue::Object( elems ) => {
    //                                 Self::handle_statements(&self, elems, collection_path)
    //                             },
    //                             _ => {
    //                                 vec![]
    //                             },
    //                         }
    //                     }).collect()
    //                 },
    //                 Some(_) => {
    //                     vec![]
    //                 },
    //                 None => vec![],
    //             }
    //         },
    //         Some(_) => vec![],
    //         None => vec![]
    //     }
    // }
}

// impl RelationshipFinder for RelationshipVisitor {
//     fn add_relationship(&self, relationship: Relationship) {
//         self.downstream_visitor.relationships.borrow_mut().push(relationship);
//     }

//     fn output_relationships(&self) -> String {
//         self.downstream_visitor.output_relationships()
//     }
// }

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
            NoIdentifiers(block) => {
                let attributes_json: Vec<String> = block.attributes.iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");

                format!(r#"{{"type":"{}","body":{{{}}}}}"#, block.block_type, attributes_joined)
            },
            WithOneIdentifier(block) => {
                let attributes_json: Vec<String> = block.attributes.iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");

                format!(r#"{{"type":"{}","name":"{}","body":{{{}}}}}"#, block.block_type, &block.first_identifier, attributes_joined)
            },
            WithTwoIdentifiers(block) => {
                match self.aws_relationship_specs.get(&block.first_identifier) {
                    Some(Relationship { source, target, label }) => {
                        let source_t_string = jmespath_query(block, source);
                        let target_t_string = jmespath_query(block, target);

                        println!("source: {:?}, target: {:?}", source_t_string, target_t_string);
                    },
                    None => print!(""),
                }

                // match self.aws_relationship_specs.get(first_identifier) {
                //     Some(Relationship::BasicRelationship { source, target, label }) => {
                        // TODO: 
                        // [ ] break up the source/target strings into their jmespath expression tokens  
                        // [ ] recursively pass through the tokens to return leaf node value  

                    //     let source_attr = attributes.into_iter().find(|&attr| attr.key == source.to_string());
                    //     let target_attr = attributes.into_iter().find(|&attr| attr.key == target.to_string());

                    //     let source_t_string = Self::extract_value(source_attr);
                    //     let target_t_string = Self::extract_value(target_attr);

                    //     fn hack_lambda_attributes(key: &String, val: String) -> String {
                    //         if key == "function_name" && !val.contains(".") {
                    //             "aws_lambda_function.".to_owned() + &val
                    //         } else {
                    //             val
                    //         }
                    //     }

                    //     if let Some(source_val) = source_t_string {
                    //         if let Some(target_val) = target_t_string {
                    //             let new_target = hack_lambda_attributes(target, target_val);
                    //             let new_source = hack_lambda_attributes(source, source_val);
                    //             let relationship = Relationship::BasicRelationship { source: new_source, target: new_target, label: String::from("") };
                    //             self.downstream_visitor.add_relationship(relationship)
                    //         }
                    //     }
                    // },
                    // Some(Relationship::NestedRelationship { source, targets: TargetAndLabel { collection_path, target, label } }) => {
                    // Some(Relationship::NestedRelationship { source, target, label } ) => {
                        // let dot_split = collection_path.split(".").collect::<Vec<&str>>();

                        // let source_attr = attributes.into_iter().find(|&attr| attr.key == source.to_string());
                        // let target_attr = attributes.into_iter().find(|&attr| attr.key == dot_split[0].to_string());

                        // let source_t_string = Self::extract_value(source_attr);

                        // let targets_and_labels = Self::extract_values(&self, target_attr, collection_path, target, label);
                        // if let Some(source_val) = source_t_string {
                        //     for target_and_label in targets_and_labels.into_iter() {
                        //         let cloned_source = source_val.clone();
                        //         let relationship = Relationship::NestedRelationship { source: cloned_source, targets: target_and_label };
                        //         self.downstream_visitor.add_relationship(relationship)
                        //     }
                        // }
                    // },
                //     None => print!(""),
                // };

                let attributes_json: Vec<String> = block.attributes.iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");
                format!(r#"{{"type":"{}","name":"{}","body":{{{}}}}}"#, block.first_identifier, block.second_identifier, attributes_joined)
            },
        };

        self.downstream_visitor.visit_tfblock(value)
    }

    fn visit_attribute(&self, value: &Attribute) -> String {
        self.downstream_visitor.visit_attribute(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn relationship_visitor_test() {
    //     let resource1 = WithOneIdentifier(
    //         TerraformBlockWithOneIdentifier {
    //             block_type: String::from("resource"),
    //             first_identifier: String::from("thing1"),
    //             attributes: vec![
    //                 Attribute {
    //                     key: String::from("backend"),
    //                     value: Str(String::from("s3"))
    //                 },
    //                 Attribute {
    //                     key: String::from("bookend"),
    //                     value: Boolean(true)
    //                 }
    //             ]
    //         }
    //     );
    //     let expected = String::from(r#"{"type":"resource","name":"thing1","body":{"backend":"s3","bookend":"true"}}"#);
        
    //     let mut vec = Vec::new();
    //     let mut h_map = HashMap::new();
    //     let visitor = RelationshipVisitor{
    //         downstream_visitor: JsonVisitor{relationships: RefCell::new(vec)},
    //         aws_relationship_specs: h_map,
    //     };  
    //     let result = visitor.visit_tfblock(&resource1);
    //     assert_eq!(result, expected)
    // }
}