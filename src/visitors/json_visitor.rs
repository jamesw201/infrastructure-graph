/// This Visitor will visit every stucture in the tree of Structs and generate json

// Useful articles on the Visitor pattern:
// https://michael-f-bryan.github.io/calc/book/html/parse/visit.html
// https://www.lihaoyi.com/post/ZeroOverheadTreeProcessingwiththeVisitorPattern.html
use std::cell::RefCell;

use crate::structs::attributes::{ Attribute, AttributeType };
use AttributeType::{
    Array, Block, Boolean, Json, Num, Str, TFBlock, TemplatedString,
};
use crate::structs::template_string::{ TemplateString };
use crate::structs::json::JsonValue;

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

use crate::relationship_finders::relationship_finder::{RelationshipFinder};
use crate::visitors::visitor::{ PolicyEvaluator, Visitor };
use crate::visitors::relationship_visitor::Relationship;


pub struct JsonVisitor {
    pub relationships: RefCell<Vec<Relationship>>,
}

impl RelationshipFinder for JsonVisitor {
    fn add_relationship(&self, relationship: Relationship) {
        self.relationships.borrow_mut().push(relationship);
    }

    fn output_relationships(&self) -> String {
        let joined_relationships = self.relationships.borrow().iter().map(|rel| format!("{}", rel)).collect::<Vec<String>>().join(",");
        format!("[{}]", joined_relationships)
    }
}

impl Visitor<String> for JsonVisitor {
    fn visit_str(&self, value: &String) -> String {
        let mut s = String::new();
        s.push_str("\"");
        let res = &value.replace("'", "");
        s.push_str(res);
        s.push_str("\"");
        s
    }

    fn visit_template_string(&self, value: &TemplateString) -> String {
        let val = match value {
            Variable(tstring) => tstring.to_owned(),
            BuiltInFunction(bif) => String::from("builtinfunction"),
        };
        let mut s = String::new();
        s.push_str("\"");
        let res = if val.contains("md5") {
            ""
        } else {
            &val
        };
        s.push_str(res);
        s.push_str("\"");
        s
    }

    fn visit_boolean(&self, value: &bool) -> String {
        let bstring = match value {
            true => "true",
            false => "false",
        };
        let mut s = String::new();
        s.push_str("\"");
        s.push_str(bstring);
        s.push_str("\"");
        s
    }

    fn visit_num(&self, value: &f64) -> String {
        let mut s = String::new();
        s.push_str("\"");
        let num_str = format!("{:.1}", &value);
        s.push_str(&num_str);
        s.push_str("\"");
        s
    }

    fn visit_block(&self, value: &Vec<Attribute>) -> String {
        let mut s = String::new();
        s.push_str("{");
        let attributes_json: Vec<String> = value.into_iter().map(|attr| self.visit_attribute(&attr)).collect();
        let attributes_joined = attributes_json.join(",");
        s.push_str(&attributes_joined.to_string());
        s.push_str("}");
        s
    }

    fn visit_array(&self, value: &Vec<AttributeType>) -> String {
        let mut s = String::new();
        let attributes_json: Vec<String> = value.into_iter().map(|attr| 
            match attr {
                Str(str_inside) => self.visit_str(str_inside),
                TemplatedString(str_inside) => self.visit_template_string(str_inside),
                Boolean(bool_inside) => self.visit_boolean(bool_inside),
                Num(num_inside) => self.visit_num(num_inside),
                Array(arr_inside) => self.visit_array(arr_inside),
                Block(block_inside) => self.visit_block(block_inside),
                TFBlock(tfblock_inside) => self.visit_tfblock(tfblock_inside),
                Json(json_inside) => self.visit_json(json_inside),
            }
        ).collect();

        s.push_str("[");
        let attributes_joined = attributes_json.join(",");
        s.push_str(&attributes_joined.to_string());
        s.push_str("]");
        s
    }

    fn visit_json_array(&self, value: &Vec<JsonValue>) -> String {
        let array_items: Vec<String> = value.into_iter().map(|val| self.visit_json(&val)).collect();
        let array_items_joined = array_items.join(",");

        format!("[{}]", &array_items_joined)
    }

    fn visit_json_object(&self, value: &Vec<(String, JsonValue)>) -> String {
        let array_items: Vec<String> = value.into_iter()
            .map(|(key, val)| {
                format!("\"{}\":{}", key, self.visit_json(&val))
            }).collect();

        let array_items_joined = array_items.join(",");

        format!("{{{}}}", &array_items_joined)
    }

    fn visit_json(&self, value: &JsonValue) -> String {
        match &value {
            JsonValue::Str(str_inside) => self.visit_str(str_inside),
            JsonValue::Boolean(bool_inside) => self.visit_boolean(bool_inside),
            JsonValue::Num(num_inside) => self.visit_num(num_inside),
            JsonValue::Null(str_inside) => self.visit_str(str_inside),
            JsonValue::Array(arr_inside) => self.visit_json_array(arr_inside),
            JsonValue::Object(obj_inside) => self.visit_json_object(obj_inside),
        }
    }

    fn visit_attribute(&self, attr: &Attribute) -> String {
        let value = match &attr.value {
            Str(str_inside) => self.visit_str(str_inside),
            TemplatedString(str_inside) => self.visit_template_string(str_inside),
            Boolean(bool_inside) => self.visit_boolean(bool_inside),
            Num(num_inside) => self.visit_num(num_inside),
            Array(arr_inside) => self.visit_array(arr_inside),
            Block(block_inside) => self.visit_block(block_inside),
            TFBlock(tfblock_inside) => self.visit_tfblock(tfblock_inside),
            Json(json_inside) => self.visit_json(json_inside),
        };

        format!("\"{}\":{}", &attr.key, value)
    }

    fn visit_tfblock(&self, value: &TerraformBlock) -> String {
        match value {
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
                let attributes_json: Vec<String> = attributes.into_iter().map(|attr| self.visit_attribute(&attr)).collect();
                let attributes_joined = attributes_json.join(",");
                format!("{{\"type\":\"{}\",\"name\":\"{}\",\"body\":{{{}}}}}", first_identifier, second_identifier, attributes_joined)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_visitor_test() {
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
        let visitor = JsonVisitor{relationships: RefCell::new(vec)};
        let result = visitor.visit_tfblock(&resource1);
        assert_eq!(result, expected)
    }

    #[test]
    fn json_visitor_json() {
        let resource1 = TerraformBlock::WithTwoIdentifiers(
            TerraformBlockWithTwoIdentifiers {
                block_type: String::from("resource"), 
                first_identifier: String::from("aws_sqs_queue"), 
                second_identifier: String::from("discovery_collector-queue"), 
                attributes: vec![
                    Attribute {
                        key: String::from("policy"),
                        value: AttributeType::Json(JsonValue::Object(vec![
                            (String::from("deadLetterTargetArn"), JsonValue::Str(String::from("${aws_sqs_queue.discovery_collector-deadletter-queue.arn}"))),
                            (String::from("maxReceiveCount"), JsonValue::Num(2.0))]))
                    },
                ]
            }
        );
        let expected = String::from(r#"{"type":"aws_sqs_queue","name":"discovery_collector-queue","body":{"policy":{"deadLetterTargetArn":"${aws_sqs_queue.discovery_collector-deadletter-queue.arn}","maxReceiveCount":"2.0"}}}"#);
        let mut vec = Vec::new();
        let visitor = JsonVisitor{relationships: RefCell::new(vec)};
        let result = visitor.visit_tfblock(&resource1);
        assert_eq!(result, expected)
    }

    #[test]
    fn json_visitor_array() {
        let resource1 = TerraformBlock::WithTwoIdentifiers(
            TerraformBlockWithTwoIdentifiers { 
                block_type: String::from("resource"),
                first_identifier: String::from("aws_cloudwatch_metric_alarm"),
                second_identifier: String::from("discovery_diff-engine-queue-cloudwatch-alaram-messages-high"),
                attributes: vec![
                    Attribute { key: String::from("alarm_name"), value: AttributeType::Str(String::from( "discovery_cloudwatch-diff-engine-queue-cloudwatch-alarm-messages-high")) },
                    Attribute { key: String::from("alarm_actions"), value: AttributeType::Array(
                        vec![
                            AttributeType::TemplatedString(TemplateString::Variable(String::from("aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn"))),
                            AttributeType::TemplatedString(TemplateString::Variable(String::from("aws_appautoscaling_policy.discovery_diff-engine-autoscaling-down.arn")))
                        ]
                    )}
                ] 
            }
        );
        let expected = String::from(r#"{"type":"aws_cloudwatch_metric_alarm","name":"discovery_diff-engine-queue-cloudwatch-alaram-messages-high","body":{"alarm_name":"discovery_cloudwatch-diff-engine-queue-cloudwatch-alarm-messages-high","alarm_actions":["aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn","aws_appautoscaling_policy.discovery_diff-engine-autoscaling-down.arn"]}}"#);
        let mut vec = Vec::new();
        let visitor = JsonVisitor{relationships: RefCell::new(vec)};
        let result = visitor.visit_tfblock(&resource1);
        assert_eq!(result, expected)
    }

    #[test]
    fn json_visitor_block_test() {
        let resource1 = TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers { 
                block_type: String::from("resource"),
                first_identifier: String::from("aws_cloudwatch_log_metric_filter"),
                second_identifier: String::from("discovery_diff-tagging-failed-event-error"),
                attributes: vec![
                    Attribute { key: String::from("name"), value: AttributeType::Str(String::from("diff_tagging_failed_event")) },
                    Attribute { key: String::from("metric_transformation"), value: AttributeType::Block(
                        vec![
                            Attribute {
                                key: String::from("name"),
                                value: AttributeType::Str(String::from("diff_tagging_failed_event"))
                            },
                            Attribute {
                                key: String::from("namespace"),
                                value: AttributeType::Str(String::from("diff_tagging_log_metrics"))
                            },
                            Attribute {
                                key: String::from("value"),
                                value: AttributeType::Str(String::from("1"))
                            }
                        ]
                    )}
                ] 
            }
        );

        let expected = String::from(r#"{"type":"aws_cloudwatch_log_metric_filter","name":"discovery_diff-tagging-failed-event-error","body":{"name":"diff_tagging_failed_event","metric_transformation":{"name":"diff_tagging_failed_event","namespace":"diff_tagging_log_metrics","value":"1"}}}"#);
        let mut vec = Vec::new();
        let visitor = JsonVisitor{relationships: RefCell::new(vec)};
        let result = visitor.visit_tfblock(&resource1);
        assert_eq!(result, expected)
    }
}
