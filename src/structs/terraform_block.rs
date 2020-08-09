use crate::structs::attributes::{ Attribute };

#[derive(PartialEq, Debug, Clone)]
pub enum TerraformBlock {
    NoIdentifiers(TerraformBlockWithNoIdentifiers),
    WithOneIdentifier(TerraformBlockWithOneIdentifier),
    WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers)
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithNoIdentifiers {
    pub block_type: String,
    pub attributes: Vec<Attribute>
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithOneIdentifier {
    pub block_type: String,
    pub first_identifier: String,
    pub attributes: Vec<Attribute>
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithTwoIdentifiers {
    pub block_type: String,
    pub first_identifier: String,
    pub second_identifier: String,
    pub attributes: Vec<Attribute>
}