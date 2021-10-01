
use crate::visitors::relationship_finder::{Relationship};

pub trait RelationshipFinder {
    fn add_relationship(&self, relationship: Relationship);
    fn output_relationships(&self) -> String;
}
