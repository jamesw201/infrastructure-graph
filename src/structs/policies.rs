use serde::{Deserialize, Serialize};
use std::clone::Clone;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Filter {
    pub key: String,
    pub op: String,
    pub value: String,
}

impl Filter {
    pub fn new(key: &str, op: &str, value: &str) -> Filter {
        Filter { key: key.to_owned(), op: op.to_owned(), value: value.to_owned() }
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
