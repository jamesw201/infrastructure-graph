#[cfg_attr(test, macro_use)] extern crate serde_json;
use structopt::StructOpt;
use std::collections::HashMap;
extern crate serde_yaml;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::time::{Instant};
use exitfailure::ExitFailure;

use rust_nom_json::structs::policies::Policies;
use rust_nom_json::*;
use rust_nom_json::visitors::resource_visitor;
use rust_nom_json::visitors::relationship_finder::{ ValidationError, Relationship };
// mod visitors;
// use visitors::relationship_finder::{ ValidationError, Relationship };
// use validator::{Validate, ValidationError};

mod terraform;

// DONE:
// [√] read this approach for mocking reqwest with traits: https://write.as/balrogboogie/testing-reqwest-based-clients
// [√] prove that you can write end-to-end unit tests from this file
// [√] create a trait for async services that return Result<Json, Error>  
// [√] use that trait to mock responses, enabling TDD for applications with side effects  
// [√] make sure that errors are transformed into some kind of standard Error

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn validate_relationships(aws_relationship_specs: &HashMap<String, Relationship>) -> Vec<ValidationError> {
    aws_relationship_specs
        .into_iter()
        .map(|(_, relationship)| relationship.validate())
        .filter(|result| result.is_err())
        .map(|result| result.unwrap_err())
        .collect()
}

fn main() -> Result<(), ExitFailure> {
    let start = Instant::now();
    let args = Cli::from_args();

    let relationship_file = std::fs::File::open("./example_files/aws_relationships.yaml")?;
    let aws_relationship_specs: HashMap<String, Relationship> = serde_yaml::from_reader(relationship_file)?;
    println!("Relationships YAML string: {:?}", aws_relationship_specs);

    // let errors = validate_relationships(&aws_relationship_specs);
    // if errors.len() > 0 {
    //     for err in &errors { 
    //         println!("Error: {:?}", err.code);
    //     }

    //     return Ok(())
    // }

    let policies_file = std::fs::File::open("./example_files/policies.yaml")?;
    let policy_specs: Policies = serde_yaml::from_reader(policies_file)?;
    // println!("Policies YAML string: {:?}", policy_specs);
    
    let occlusions_file = std::fs::File::open("./example_files/occlusions.yaml")?;
    let occlusion_specs: HashMap<String, Vec<String>> = serde_yaml::from_reader(occlusions_file)?;
    println!("Occlusions YAML string: {:?}", occlusion_specs);

    let parser = cloud_template_parser::CloudTemplateParser::new();
    let parsed_resources = parser.handle(args.path);

    let json = resource_visitor::dispatch(&parsed_resources, aws_relationship_specs, policy_specs, occlusion_specs);
    // // iterate over array, use match statement to get initial visitor right
    // // then allow Visitor pattern to do the rest
    let elapsed_before_printing = start.elapsed();

    let path = Path::new("graph.json");
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write json to file
    match file.write_all(json.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
    // // println!("json: {:?}", json);
    // // println!("ast: {:?}", parsed_resources);

    let duration = start.elapsed();
    println!("Terraform parsed in: {:?}", elapsed_before_printing);
    println!("Terraform printed to stdout in: {:?}", duration);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use anyhow::Result;
    use serde_json::Value;
    // use std::io::BufReader;
    // use std::fs::File;

    // TODO: test that library can read from an array of  BufferedReader
    // depending on the filename { bla.tfvars, bla.variables, bla.tf } we will look to either parse or create values for string interpolation.
    // #[test]
    // fn buffer_reader_test() {
    //     struct FileType<T> {
    //         filename: String,
    //         reader: BufReader<T>,
    //     };
    //     impl <T> FileType<T> {
    //         pub fn new(filename: String, reader: BufReader<T>) -> FileType<T> {
    //             FileType { filename, reader }
    //         }
    //     }

    //     // 
    //     let tf_file = File::open("fake.tf").unwrap();
    //     let tfvars_file = File::open("fake.tfvars").unwrap();
    //     let variables_file = File::open("variables.tf").unwrap();

    //     let tf = FileType::new(String::from("fake.tf"), BufReader::new(tf_file));
    //     let tfvars = FileType::new(String::from("fake.tfvars"), BufReader::new(tfvars_file));
    //     let variables = FileType::new(String::from("variables.tf"), BufReader::new(variables_file));

    //     let files = vec![tf, tfvars, variables];
    //     let parser = cloud_template_parser::CloudTemplateParser::new();
    //     let result = parser.handle(vec![String::from("discovery.tf")]);
    //     assert_eq!(true, true)

    //     // assert_eq!(result, json!({
    //     //     "resources": [
    //     //         {
    //     //             "resource": "bla",
    //     //             "foo": "bar",
    //     //             "baz": "quux",
    //     //         },
    //     //         {
    //     //             "resource": "bloop",
    //     //             "foo": "bar",
    //     //             "baz": "quux",
    //     //         }
    //     //     ],
    //     //     "relationships": [
    //     //         {
    //     //             "source": "bla",
    //     //             "target": "bloop",
    //     //         }
    //     //     ]
    //     // }));
    // }
}
