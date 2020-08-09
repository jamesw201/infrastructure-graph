extern crate nom;
use nom::{
  IResult,
};
// use crate::terraform::root;
use crate::terraform::{
  root,
};

use crate::structs::terraform_block::{
    TerraformBlock,
};

use std::fs;

/// CloudTemplateParser -> (ResourceTree)
/// - reads in (Terraform, Cloudformation) templates -- might be replaced by separate FileReader Entity at some point.
/// - returns a ResourceTree representing all of the resources in the CloudTemplate
/// - uses nom to create a ResourceTree.

#[derive(Debug)]
pub struct ResourceTree {}

impl ResourceTree {
    pub fn new() -> ResourceTree {
        ResourceTree {}
    }
}

#[derive(Debug)]
pub struct CloudTemplateParser {}

impl CloudTemplateParser {
    pub fn new() -> CloudTemplateParser {
        CloudTemplateParser {}
    }

    pub fn handle(&self, filename: std::path::PathBuf) -> Vec<TerraformBlock> {
        // let contents = fs::read_to_string("example_files/discovery-minus-bad-bits.tf")
        //     .expect("Something went wrong reading the file");
        let contents = fs::read_to_string(filename)
            .expect("Something went wrong reading the file");

        let (_, result) = root(contents.as_str()).unwrap();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn parse_test() {
    //     let parser = CloudTemplateParser::new();
    //     let filenames = vec![String::from("./example_files/discovery.tf"), String::from("./example_files/discovery.tfvars")];
    //     let result = parser.handle(filenames);
    //     println!("ast: {:?}", result);
    //     let expected = ResourceTree::new();
    //     assert_eq!(1, 2)
    //     // assert_eq!(result, expected)
    // }
}
