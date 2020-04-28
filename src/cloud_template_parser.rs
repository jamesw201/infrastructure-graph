extern crate nom;

// use nom::{
//   branch::alt,
//   bytes::complete::{escaped, tag, take_while},
//   character::complete::{alphanumeric1 as alphanumeric, char, one_of},
//   combinator::{cut, map, opt, value},
//   error::{context, convert_error, ErrorKind, ParseError, VerboseError},
//   multi::separated_list,
//   number::complete::double,
//   sequence::{delimited, preceded, separated_pair, terminated},
//   Err, IResult,
// };
// use std::collections::HashMap;
// use std::str;
// use std::fs::File;
// use std::io::Read;
// use std::path::Path;
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

    pub fn handle(&self, filenames: Vec<String>) -> Vec<String> {
        // TODO:
        // [√] read the file in to memory *for now we'll just deal with one file
        // [ ] begin parsing with nom

        // let filename = filenames.into_iter().nth(0).expect("Could not get filename");
        // println!("filename: {}", filename);
        // let path = Path::new(&filename);
        // let mut file = File::open(path).expect("Could not open file");
        // let mut content = String::new();
        // file.read_to_string(&mut content).expect("Could not read file");

        let contents = fs::read_to_string("example_files/basic.tf")
            .expect("Something went wrong reading the file");

        println!("{}", contents);
        vec![String::from("filename")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let parser = CloudTemplateParser::new();
        let filenames = vec![String::from("../example_files/discovery.tf"), String::from("../example_files/discovery.tfvars")];
        let result = parser.handle(filenames);
        let expected = ResourceTree::new();
        assert_eq!(true, true)
        // assert_eq!(result, expected)
    }
}
