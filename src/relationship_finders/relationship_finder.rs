
use crate::visitors::relationship_visitor::{Relationship};

pub trait RelationshipFinder {
    fn add_relationship(&self, relationship: Relationship);
    fn output_relationships(&self) -> String;
}
