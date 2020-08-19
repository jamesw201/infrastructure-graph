use serde::{Deserialize, Serialize};
use std::clone::Clone;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Filter {
    pub key: String,
    pub op: String,
    pub value: String,
}

impl Clone for Filter {
    fn clone(&self) -> Filter {
        Filter { key: self.key.clone(), op: self.op.clone(), value: self.value.clone() }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Policy {
    pub name: String,
    pub description: String,
    pub resource: String,
    pub filters: Vec<Filter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
