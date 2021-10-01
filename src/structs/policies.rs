use std::fmt;
use serde::{Deserialize, Serialize};
use std::clone::Clone;

use crate::relationship_finders::tf_block_query::tf_block_query::TFQueryResult;
use crate::structs::attributes::AttributeType;
use AttributeType::{ Block, Str, Num };

use TFQueryResult::{ List, Scalar };

pub enum Operation {
    Eq(),
    Contains(),
    LessThan(),
    GreaterThan(),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FilterResult {
    pub filter: Filter, // TODO: this should be a Vector of Filters
    pub result: bool,
}

impl FilterResult {
    pub fn new(filter: Filter, result: bool) -> FilterResult {
        FilterResult { filter, result }
    }
}

impl fmt::Display for FilterResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"{{"filter":"{}","result":"{}"}}"#, self.filter, self.result)
    }
}

// TODO: formalise each attribute into proper types with constraints
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Filter {
    pub key: String,    // a non null string
    pub op: String,     // should be initialised as a valid input for an Operation
    pub value: String,  // a non null string
}

impl Filter {
    pub fn new(key: &str, op: &str, value: &str) -> Filter {
        Filter { key: key.to_owned(), op: op.to_owned(), value: value.to_owned() }
    }

    fn match_value(operator: &str, filter_value: &String, query_result_value: &AttributeType) -> bool {
        match operator {
            "eq" => {
                match query_result_value {
                    Block(attributes) => {
                        if &attributes.len() > &0 {
                            match &attributes[0].value {
                                Str(val) => val == filter_value,
                                Num(val) => val == &filter_value.parse::<f64>().unwrap(),
                                // TODO: accommodate all variants
                                _ => false,
                            }
                        } else {
                            // TODO: throw custom error
                            false
                        }
                    },
                    Str(val) => val == filter_value,
                    Num(val) => val == &filter_value.parse::<f64>().unwrap(),
                    _ => false,
                }
            },
            "lt" => {
                match query_result_value {
                    Str(val) => val < filter_value,
                    Num(val) => val < &filter_value.parse::<f64>().unwrap(),
                    _ => false,
                }
            },
            "gt" => {
                match query_result_value {
                    Str(val) => val > filter_value,
                    Num(val) => val > &filter_value.parse::<f64>().unwrap(),
                    _ => false,
                }
            },
            // TODO: Should return unknown operator Error
            _ => false,
        }
    }

    pub fn evaluate(&self, query_result: TFQueryResult) -> FilterResult {
        let result = match &query_result {
            Scalar(attr_type) => Self::match_value(self.op.as_str(), &self.value, attr_type),
            List(attr_types) => {
                match self.op.as_str() {
                    "eq" => {
                        attr_types.iter()
                            .map(|attr| Self::match_value("eq", &self.value, attr))
                            .fold(true, |res, val| res && val)
                    },
                    "contains" => attr_types.iter().any(|attr| Self::match_value("eq", &self.value, attr)),
                    _ => false,
                }
            },
            TFQueryResult::None => {
                false
            },
        };

        FilterResult::new(self.clone(), result)
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"key\":\"{}\",\"op\":\"{}\",\"value\":\"{}\"}}", self.key, self.op, self.value)
    }
}

impl Clone for Filter {
    fn clone(&self) -> Filter {
        Filter { key: self.key.clone(), op: self.op.clone(), value: self.value.clone() }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct Policy {
    pub name: String,
    pub description: String,
    pub resource: String,
    pub filters: Vec<Filter>,
}

impl Policy {
    pub fn new(name: &str, description: &str, resource: &str, filters: Vec<Filter>) -> Policy {
        Policy { name: name.to_owned(), description: description.to_owned(), resource: resource.to_owned(), filters }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Policies {
    pub policies: Vec<Policy>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PolicyResult {
    policy: Policy,
    passed: bool,
    resource_id: String,
    linked_resource_id: String,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::attributes::{
        Attribute,
        AttributeType,
    };

    #[test]
    fn evaluate_filter_test() {
        let attribute_type = AttributeType::Block(vec![Attribute{ key: String::from("policy.maxReceiveCount"), value: AttributeType::Num(2.0) }]);
        let query_result = TFQueryResult::Scalar(attribute_type);
        
        let filter = Filter::new("policy.maxReceiveCount", "eq", "2.0");
        let result = filter.evaluate(query_result);
        assert_eq!(result, FilterResult::new(filter, true))
    }
}