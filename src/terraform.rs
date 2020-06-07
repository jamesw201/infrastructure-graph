extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{escaped, is_not, tag, take, take_while, take_until},
  character::complete::{alphanumeric1, char, one_of, multispace0, newline, not_line_ending, line_ending, space0, space1},
  combinator::{map, opt, peek, value},
  error::{ParseError},
  multi::{many0, many1, separated_list0, fold_many0},
  number::complete::double,
  sequence::{delimited, preceded, pair, separated_pair, terminated},
  IResult
};

use crate::json::{JsonValue, parse_json};

use std::str;

// #[derive(PartialEq, Debug, Clone)]
// pub enum BuiltInFunctionParam {
//     Path(String),
//     InnerFunction(Box<BuiltInFunction>),
// }

#[derive(PartialEq, Debug, Clone)]
pub struct BuiltInFunction {
    name: String,
    param: TemplateString,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TemplateString {
    Variable(String),
    BuiltInFunction(Box<BuiltInFunction>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum AttributeType {
    Str(String),
    TemplatedString(TemplateString),
    Boolean(bool),
    Num(f64),
    Array(Vec<AttributeType>),
    Block(Vec<(String, AttributeType)>),
    TFBlock(TerraformBlock),
    Json(JsonValue),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Attribute {
    key: String,
    value: AttributeType
}

#[derive(PartialEq, Debug, Clone)]
pub enum TerraformBlock {
    NoIdentifiers(TerraformBlockWithNoIdentifiers),
    WithOneIdentifier(TerraformBlockWithOneIdentifier),
    WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers)
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithNoIdentifiers {
    block_type: String,
    attributes: Vec<Attribute>
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithOneIdentifier {
    block_type: String,
    first_identifier: String,
    attributes: Vec<Attribute>
}

#[derive(PartialEq, Debug, Clone)]
pub struct TerraformBlockWithTwoIdentifiers {
    block_type: String,
    first_identifier: String,
    second_identifier: String,
    attributes: Vec<Attribute>
}


// #[allow(dead_code)]
// fn take_and_consume(i: &str) -> IResult<&str, &str> {
//     let (res, _) = take_until("}")(i)?;
//     take(1usize)(res)
// }

#[allow(dead_code)]
fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
  let parse_true = value(true, tag("true"));
  let parse_false = value(false, tag("false"));

  alt((parse_true, parse_false))(input)
}

fn parse_single_line_str(i: &str) -> IResult<&str, &str> {
    preceded(space0, escaped(is_not("\""), '\\', one_of(r#"\"#)))(i)
    // take_while(is_not_newline)(i)
}

// fn parse_str(i: &str) -> IResult<&str, &str> {
//     preceded(space0, escaped(is_not("\\\n"), '\\', one_of(r#"\"#)))(i)
// }

fn parse_array(i: &str) -> IResult<&str, Vec<AttributeType>> {
    let (rest, result) = preceded(
        char('['),
        preceded(
            multispace0,
            terminated(
                separated_list0(preceded(space0, char(',')), preceded(multispace0, block_value)),
                preceded(multispace0, char(']'))
            )
        )
    )(i)?;
    // println!("parse_array: {}", i);
    Ok((rest, result))
}

fn valid_identifier_char(c: char) -> bool {
  c.is_alphanumeric() || c == '_' || c == '/'
}

fn valid_identifier(i: &str) -> IResult<&str, &str> {
    take_while(valid_identifier_char)(i)
}

#[allow(dead_code)]
fn key_value(i: &str) -> IResult<&str, (&str, AttributeType)> {
    let (rest, result) = preceded(
        multispace0,
        alt((
            separated_pair(
                preceded(
                    preceded(space0, char('\"')), 
                    terminated(
                        valid_identifier, 
                        char('\"')
                    )
                ),
                preceded(
                    space0, 
                    char('=')
                ),
                block_value
            ),
            separated_pair(preceded(space0, valid_identifier), space0, block_value),
            separated_pair(preceded(space0, valid_identifier), preceded(space0, char('=')), block_value),
        ))
    )(i)?;

    Ok((rest, result))
}

fn json_value(i: &str) -> IResult<&str, JsonValue> {
    let (rest, result) = preceded(
        multispace0, 
        preceded(
            alt((
                tag("<<EOF"),
                tag("<<CONTAINER_DEFINITIONS")
            )), 
            terminated(
                parse_json, 
                preceded(
                    multispace0, 
                    alt((
                        tag("EOF"),
                        tag("CONTAINER_DEFINITIONS")
                    ))
                )
            )
        )
    )(i)?;

    Ok((rest, result))
}

fn serialised_json(i: &str) -> IResult<&str, JsonValue> {
    delimited(tag("\""), parse_json, tag("\""))(i)
}

fn block_value(i: &str) -> IResult<&str, AttributeType> {
  preceded(
    space0,
    alt((
      map(templated_string, AttributeType::TemplatedString),
      map(serialised_json, AttributeType::Json),
      map(boolean, AttributeType::Boolean),
      map(double, AttributeType::Num),
      map(basic_block, AttributeType::Block),
      map(tf_block, AttributeType::TFBlock),
      map(escaped_string, |s| AttributeType::Str(String::from(s))),
      map(parse_array, AttributeType::Array),
      map(json_value, AttributeType::Json),
    )),
  )(i)
}

fn escaped_string(i: &str) -> IResult<&str, &str> {
    let (rest, result) = preceded(char('\"'), terminated(parse_single_line_str, char('\"')))(i)?;
    // if result != "" {
    //     println!("returning string: {:?}", result);
    // }
    Ok((rest, result))
}

fn separated_attributes(i: &str) -> IResult<&str, Vec<(&str, AttributeType)>> {
    separated_list0(preceded(space0, newline), key_value)(i)
}

fn build_tf_block(identifiers: Vec<&str>, attributes: Vec<(String, AttributeType)>) -> TerraformBlock {
    match identifiers.len() {
        1 => {
            let block_types = ["resource", "provider", "data", "terraform", "variable"];
            if block_types.contains(&identifiers[0]) {
                TerraformBlock::NoIdentifiers(
                    TerraformBlockWithNoIdentifiers {
                        block_type: identifiers[0].to_string(),
                        attributes: attributes.into_iter().map(|(key, value)| Attribute{key, value}).collect()
                    }
                )
            } else {
                TerraformBlock::WithOneIdentifier(
                    TerraformBlockWithOneIdentifier {
                        block_type: String::from("inner"),
                        first_identifier: identifiers[0].to_string(),
                        attributes: attributes.into_iter().map(|(key, value)| Attribute{key, value}).collect()
                    }
                )
            }
        },
        2 => {
            TerraformBlock::WithOneIdentifier(
                TerraformBlockWithOneIdentifier {
                    block_type: identifiers[0].to_string(),
                    first_identifier: identifiers[1].to_string(),
                    attributes: attributes.into_iter().map(|(key, value)| Attribute{key, value}).collect()
                }
            )
        },
        3 => {
            TerraformBlock::WithTwoIdentifiers(
                TerraformBlockWithTwoIdentifiers {
                    block_type: identifiers[0].to_string(),
                    first_identifier: identifiers[1].to_string(),
                    second_identifier: identifiers[2].to_string(),
                    attributes: attributes.into_iter().map(|(key, value)| Attribute{key, value}).collect()
                }
            )
        },
        _ => panic!("encountered terraform block with too many identifiers: {:?}", identifiers)
    }
}

fn inline_block(i: &str) -> IResult<&str, (&str, AttributeType)> {
    delimited(preceded(space0, tag("{")), key_value, preceded(space0, tag("}")))(i)
}

fn basic_block(i: &str) -> IResult<&str, Vec<(String, AttributeType)>> {
    preceded(
        char('{'),
        preceded(
            multispace0,
            terminated(
                map(
                    separated_attributes, 
                        |tuple_vec| { // tempted to see if we can do without this mapping
                            tuple_vec.into_iter().map(|(k, v)| (String::from(k), v)).collect()
                        }
                    ),
                preceded(multispace0, char('}')),
            )
        )
    )(i)
}

fn parse_identifiers(i: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        multispace0,
        many0(
            preceded(space0, alt((alphanumeric1, escaped_string)))
        ),
        opt(space0)
    )(i)
}

fn tf_block(i: &str) -> IResult<&str, TerraformBlock> {
    // println!("potential tf block: {}", i);
    let (rest, _) = comments_and_blank_lines(i)?;
    let (rest, identifiers) = parse_identifiers(rest)?;
    // println!("identifiers: {:?}", identifiers);

    let (rest, attributes) = basic_block(rest)?;

    // println!("attributes: {:?}", attributes);

    let block = build_tf_block(identifiers, attributes);
    Ok((rest, block))
}

fn string_nl(i: &str) -> IResult<&str, &str> {
    let chars = "\r\n";
    take_while(move |c| !chars.contains(c))(i)
}

#[allow(dead_code)]
fn comment_one_line(i: &str) -> IResult<&str, &str> {
    preceded(
        alt((tag("//"), tag("#"))), 
        string_nl
    )(i)
}

fn sp(i: &str) -> IResult<&str, &str> {
  let chars = " \t\r\n";
  take_while(move |c| chars.contains(c))(i)
}

fn comments_and_blank_lines(i: &str) -> IResult<&str, &str> {
    let (rest, result) = alt((
        comment_one_line,
        multispace0,
    ))(i)?;

    let (_, n_result) = peek(take(1usize))(rest)?;

    if n_result == "#" || n_result == "\n" {
        comments_and_blank_lines(rest)
    } else {
        Ok((rest, result))
    }
}

#[allow(dead_code)]
pub fn root(i: &str) -> IResult<&str, Vec<TerraformBlock>> {
    many0(
        tf_block
    )(i)
}

// TODO: finish this idea!
// all things that can exist in a templated string ${}
// fn templated_value(i: &str) -> IResult<&str, TemplateString> {
//     alt((
//         map(built_in_function, TemplateString::BuiltInFunction),
//         map(valid_template_string, TemplateString::Variable),
//     ))(i)
// }

fn built_in_function(i: &str) -> IResult<&str, TemplateString> {
    // println!("built_in_function input: {}", i);

    // peek for brackets
    let (rest, result) = many0(
            pair(
            alphanumeric1, 
            delimited(
                char('('), 
                valid_template_string,
                char(')')
            ),
        )
    )(i)?;
    // TODO: use the same pattern that block_value uses to limit number 
    // of things we can search for in a recursive structure

    // println!("result.0 type: {:?}, result.1 type: {:?}", result.0.type_name(), result.1.type_name());
    // println!("built_in_function rest: {:?}, result: {:?}", rest, result);

    // let blarp = if i.contains("(") {
    //     let blap: BuiltInFunction = result.1;

    //     BuiltInFunction {
    //         name: String::from(result.0),
    //         param: TemplateString::BuiltInFunction(Box::new(blap))
    //     };
    // } else {
    //     let bleep: TemplateString = result.1;
    //     println!("found string: {:?}", bleep);
    // };

    let blarp = TemplateString::Variable(String::from(""));
    Ok((rest, blarp))
}

fn valid_template_string_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '/' || c == '.' || c == '(' || c == ')' || c == '\"'
}

fn valid_template_string(i: &str) -> IResult<&str, TemplateString> {
    // println!("valid_template_string input: {}", i);
    // let (rest, result) = delimited(tag("\""), take_while(valid_template_string_char), tag("\""))(i)?;
    let (rest, result) = take_while(valid_template_string_char)(i)?;
    let blarp = TemplateString::Variable(String::from(result));

    // println!("string result: {:?}", result);
    // Ok((rest, result))
    Ok((rest, blarp))
}

fn templated_string(i: &str) -> IResult<&str, TemplateString> {
    // println!("templated_string: {:?}", i);

    let (rest, result) = preceded(
        preceded(space0, tag("\"${")),
        terminated(valid_template_string, tag("}\""))
    )(i)?;
    
    // println!("templated rest: {:?}, result: {:?}", rest, result);
    Ok((rest, result))
}

// TODO: 
// [√] parse multiple resources separated by blank lines
// [√] parse multiple resources separated by blank lines and comment lines
// [√] parse nested blocks
// [√] parse arrays
// [√] parse nested json blocks
// [√] parse serialised json blocks
// [√] handle these: request_templates = { "application/json" = "{ \"statusCode\": 200 }" }
// [ ] parse templated strings
// [ ] handle these: etag              = "${md5(file("default-config/cpsc-vmware-config.json"))}"
// [ ] parse whole files from cli
// [ ] build relationships from templated attribute values
// [ ] build relationships json values

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templated_string_test() {
        let data = "${someVariable}";
        let expected = TemplateString::Variable(String::from("someVariable"));
        let result = templated_string(data).unwrap();

        assert_eq!(result, ("}", expected))
    }

    #[test]
    fn built_in_function_test() {
        // TODO: create stuct for built-in function that can contain an enum which is either 
        // a built-in function or a string
        let data = r#""${md5(file("/Users/james.n.wilson/code/work/repos/cd-pipeline/../service-discovery//infrastructure/default-config/cpsc-vmware-config.json"))}""#;
        let result = templated_string(data).unwrap();
        assert_eq!(1, 2)
    }

    #[test]
    fn built_in_function_in_resource() {
        // TODO: create stuct for built-in function that can contain an enum which is either 
        // a built-in function or a string
        let data = r#"
resource "aws_s3_bucket_object" "discovery_cpsc-vmware-config" {
    bucket               = "acp-platform-s-discovery-sandbox1"
    source               = "/Users/james.n.wilson/code/work/repos/cd-pipeline/../service-discovery//infrastructure/default-config/cpsc-vmware-config.json"
    etag                 = "${md5(file("/Users/james.n.wilson/code/work/repos/cd-pipeline/../service-discovery//infrastructure/default-config/cpsc-vmware-config.json"))}"
    key                  = "default-config/cpsc-vmware-config.json"
}
"#;
        let result = root(data).unwrap();
        assert_eq!(1, 2)
    }

    #[test]
    fn inline_block_with_one_pair() {
        let data = r#"{ "application/json" = "{ \"statusCode\": 200 }" }"#;
        let result = inline_block(data);
        let expected = Ok(("", ("application/json", AttributeType::Json(JsonValue::Object(vec![(String::from("statusCode"), JsonValue::Num(200.0))])))));
        assert_eq!(result, expected)
    }

    #[test]
    fn inline_block_resource() {
        let data = r#"
resource "aws_api_gateway_integration" "discovery_thing" {
    request_templates = { "application/json" = "{ \"statusCode\": 200 }" }
}
"#;
        let (_, result) = root(data).unwrap();
        // println!("inline_block_resource: {:?}", result);
        let expected = vec![TerraformBlock::WithTwoIdentifiers(
            TerraformBlockWithTwoIdentifiers {
                block_type: String::from("resource"),
                first_identifier: String::from("aws_api_gateway_integration"),
                second_identifier: String::from("discovery_thing"),
                attributes: vec![
                    Attribute {
                        key: String::from("request_templates"),
                        value: AttributeType::Block(vec![
                            (String::from("application/json"), AttributeType::Json(JsonValue::Object(vec![(String::from("statusCode"), JsonValue::Num(200.0))])))
                        ]
                    )
                }]
            }
        )];
        assert_eq!(result, expected)
    }

    #[test]
    fn parse_serialised_policy_strings() {
        let data = r#"
resource "aws_sqs_queue" "discovery_collector-queue" {
    visibility_timeout_seconds = 30
    policy               = "{\"deadLetterTargetArn\":\"${aws_sqs_queue.discovery_collector-deadletter-queue.arn}\",\"maxReceiveCount\":2}"
    tags {
      Environment        = "sandbox1"
    }
}
"#;
        let (rest, result) = root(data).unwrap();
        let expected = vec![TerraformBlock::WithTwoIdentifiers(
            TerraformBlockWithTwoIdentifiers {
                block_type: String::from("resource"), 
                first_identifier: String::from("aws_sqs_queue"), 
                second_identifier: String::from("discovery_collector-queue"), 
                attributes: vec![
                    Attribute {
                        key: String::from("visibility_timeout_seconds"), 
                        value: AttributeType::Num(30.0)
                    },
                    Attribute {
                        key: String::from("policy"),
                        value: AttributeType::Json(JsonValue::Object(vec![
                            (String::from("deadLetterTargetArn"), JsonValue::Str(String::from("${aws_sqs_queue.discovery_collector-deadletter-queue.arn}"))),
                            (String::from("maxReceiveCount"), JsonValue::Num(2.0))]))
                    },
                    Attribute {
                        key: String::from("tags"), value: AttributeType::Block(vec![(String::from("Environment"), AttributeType::Str(String::from("sandbox1")))])
                    }
                ]
            }
        )];

        assert_eq!(result, expected)
    }

    #[test] 
    fn terraform_block_with_empty_line_comments() {
        let data = r#"
# a line of comments to be ignored
#
# another line of comments to be ignored

# some other lines of comments
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}
"#;
        let result = root(data);
        let first_attr = Attribute {
            key: String::from("description"),
            value: AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))
        };
        let second_attr = Attribute {
            key: String::from("enable_key_rotation"),
            value: AttributeType::Boolean(true)
        };
        let block = TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_kms_key"),
            second_identifier: String::from("discovery_cache-master-key"),
            attributes: vec![first_attr, second_attr]
        };
        let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
        assert_eq!(result, Ok(("\n", expected)))
    }

    #[test] 
    fn terraform_block_with_comments() {
        let data = r#"
# a line of comments to be ignored
# another line of comments to be ignored
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}
"#;
        let result = root(data);
        let first_attr = Attribute {
            key: String::from("description"),
            value: AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))
        };
        let second_attr = Attribute {
            key: String::from("enable_key_rotation"),
            value: AttributeType::Boolean(true)
        };
        let block = TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_kms_key"),
            second_identifier: String::from("discovery_cache-master-key"),
            attributes: vec![first_attr, second_attr]
        };
        let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
        assert_eq!(result, Ok(("\n", expected)))
    }

    #[test] 
    fn terraform_block_preceded_by_comments_and_blank_lines() {
        let data = r#"
# ==============================================
# Imported from Terraform files

# ----------------------------------------------------------
# Master key used for encrypting/decrypting discovery collector
# cache tokens
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}
"#;
        let result = root(data);
        let first_attr = Attribute {
            key: String::from("description"),
            value: AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))
        };
        let second_attr = Attribute {
            key: String::from("enable_key_rotation"),
            value: AttributeType::Boolean(true)
        };
        let block = TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_kms_key"),
            second_identifier: String::from("discovery_cache-master-key"),
            attributes: vec![first_attr, second_attr]
        };
        let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
        assert_eq!(result, Ok(("\n", expected)))
        // assert_eq!(1, 2)
    }

//     #[test]
//     fn case_study() {
//         let data = r#"
// resource "aws_ecs_service" "discovery_collector-service" {
//   name = "discovery_collector-service"
//   cluster = "${aws_ecs_cluster.discovery_platform-s-cluster.id}"
//   task_definition = "${aws_ecs_task_definition.discovery_collector-task-definition.arn}"
//   desired_count = 1
//   launch_type = "FARGATE"
//   deployment_minimum_healthy_percent = 10
//   deployment_maximum_percent = 200
//   network_configuration = {
//     subnets = [
//       "${aws_subnet.discovery_private-subnet-az-a.id}",
//       "${aws_subnet.discovery_private-subnet-az-b.id}"
//     ]
//     security_groups = [
//       "${aws_security_group.discovery_cache-security-group1.id}"
//     ]
//   }
//   depends_on = ["aws_iam_role.discovery_ecs-execution-role"]
// }
// "#;
//         let (_, result) = root(data).unwrap();
//         println!("case study: {:?}", result);
//         assert_eq!(1, 2)
//     }

    #[test]
    fn string_test() {
        let data = r#""= 0.11.2"
backend "s3" {
    bucket  = "acp-platform-s-discovery-sandbox1"
    key     = "infrastructure/terraform.tfstate"
    region  = "us-east-1"
  }
}"#;
        let result = escaped_string(data).unwrap();
        assert_eq!(result, (r#"
backend "s3" {
    bucket  = "acp-platform-s-discovery-sandbox1"
    key     = "infrastructure/terraform.tfstate"
    region  = "us-east-1"
  }
}"#, "= 0.11.2"))
    }


//     #[test]
//     fn container_json_test() {
//         let data = r#"
// # ==============================================
// # discovery ECS Task Definition
// # ==============================================
// resource "aws_ecs_task_definition" "discovery_diff-engine-task-definition" {
//   depends_on = ["aws_elasticache_replication_group.discovery_cache-replication-group1"]
//   container_definitions = <<CONTAINER_DEFINITIONS
// [
//   {
//     "name": "discovery_diff-engine",
//     "image": "309983114184.dkr.ecr.us-east-1.amazonaws.com/acp-platform-s/discovery_diff-engine:latest",
//     "portMappings": [],
//     "ulimits": [
//       {
//         "name": "nofile",
//         "hardLimit": 100000,
//         "softLimit": 100000
//       }
//     ],
//     "environment": [
//       {
//         "name": "AUTHENTICATION_KEY",
//         "value": "discovery_diff-engine_api_key"
//       }
//     ],
//     "logConfiguration": {
//       "logDriver": "awslogs",
//       "options": {
//         "awslogs-group": "/aws/fargate/discovery_diff-engine",
//         "awslogs-region": "us-east-1"
//       }
//     },
//     "entryPoint": null,
//     "essential": true
//   }
// ]
// CONTAINER_DEFINITIONS
// }
// "#;
//         let result = root(data).unwrap();
//         println!("container result: {:?}", result);
//         assert_eq!(1,2)
//     }

    #[test]
    fn nested_block_with_an_identifier() {
        let data = r#"
terraform {
  required_version = "= 0.11.2"
  backend "s3" {
    bucket  = "acp-platform-s-discovery-sandbox1"
    key     = "infrastructure/terraform.tfstate"
    region  = "us-east-1"
  }
}
"#;
        let (_, result) = root(data).unwrap();
        let block = TerraformBlockWithNoIdentifiers { 
            block_type: String::from("terraform"),
            attributes: vec![
                Attribute { key: String::from("required_version"), value: AttributeType::Str(String::from("= 0.11.2")) },
                Attribute { key: String::from("backend"), value: AttributeType::TFBlock(TerraformBlock::WithOneIdentifier(TerraformBlockWithOneIdentifier {
                    block_type: String::from("inner"),
                    first_identifier: String::from("s3"),
                    attributes: vec![
                        Attribute { key: String::from("bucket"), value: AttributeType::Str(String::from("acp-platform-s-discovery-sandbox1")) },
                        Attribute { key: String::from("key"), value: AttributeType::Str(String::from("infrastructure/terraform.tfstate")) },
                        Attribute { key: String::from("region"), value: AttributeType::Str(String::from("us-east-1")) }
                    ]
                }))}
            ]
        };

        let expected = vec![TerraformBlock::NoIdentifiers(block)];
        assert_eq!(result, expected)
    }

//     #[test]
//     fn parse_comments_and_blank_lines() {
//         let data = r#"
// // # ==============================================
// // # Imported from Terraform files

// // # ----------------------------------------------------------
// // # Master key used for encrypting/decrypting discovery collector
// // # cache tokens
// resource "aws_iam_role" "discovery_vpc-flow-log-role" {
//    name = "discovery_vpc-flow-log-role"
// }
// "#;
//         let (rest, result) = comments_and_blank_lines(data).unwrap();
//         println!("test final rest: {}, result: {:?}", rest, result);
//         assert_eq!(rest, r#"resource "aws_iam_role" "discovery_vpc-flow-log-role" {
//    name = "discovery_vpc-flow-log-role"
// }
// "#)
//     }

//     #[test]
//     fn parse_embedded_json() {
//         let data = r#"
// resource "aws_iam_role" "discovery_vpc-flow-log-role" {
//   name = "discovery_vpc-flow-log-role"

//   assume_role_policy = <<EOF
// {
//   "Version": "2012-10-17",
//   "Statement": [
//     {
//       "Sid": "",
//       "Effect": "Allow",
//       "Principal": {
//         "Service": "vpc-flow-logs.amazonaws.com"
//       },
//       "Action": "sts:AssumeRole"
//     }
//   ]
// }
// EOF
// }
// "#;
//         let (_, result) = root(data).unwrap();
//         let block = TerraformBlockWithTwoIdentifiers { 
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_iam_role"),
//             second_identifier: String::from("discovery_vpc-flow-log-role"),
//             attributes: vec![
//                 Attribute { key: String::from("name"), value: AttributeType::Str(String::from("discovery_vpc-flow-log-role")) },
//                 Attribute { key: String::from("assume_role_policy"), value: AttributeType::Json(
//                     JsonValue::Object(
//                         vec![
//                             (String::from("Version"), JsonValue::Str(String::from("2012-10-17"))),
//                             (String::from("Statement"), JsonValue::Array(
//                                 vec![
//                                     JsonValue::Object(
//                                         vec![
//                                             (String::from("Sid"), JsonValue::Str(String::from(""))), 
//                                             (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
//                                             (String::from("Principal"), JsonValue::Object(
//                                                 vec![
//                                                     (String::from("Service"), JsonValue::Str(String::from("vpc-flow-logs.amazonaws.com")))
//                                                 ],
//                                             )),
//                                             (String::from("Action"), JsonValue::Str(String::from("sts:AssumeRole")))
//                                         ]
//                                     )
//                                 ])
//                             ),
//                         ]
//                     )
//                 )}
//             ] 
//         };
//         let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
//         assert_eq!(result, expected)
//     }

//     #[test]
//     fn array() {
//         let data = "[\"${aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn}\"]";
//         let (_, result) = parse_array(data).unwrap();
//         let expected = vec![
//             AttributeType::Str(String::from("${aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn}"))
//         ];
//         assert_eq!(result, expected)
//     }

//     #[test]
//     fn key_value_escaped_string() {
//         let data = r#"description = "Master key used for creating/decrypting cache token data keys""#;
//         let (_, res) = key_value(data).unwrap();
//         println!("res: {:?}", res);
//         assert_eq!(res, ("description", AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))))
//     }

//     #[test]
//     fn key_value_snake_case_identifier() {
//         let data = r#"enable_key_rotation = true"#;
//         let (_, res) = key_value(data).unwrap();
//         assert_eq!(res, ("enable_key_rotation", AttributeType::Boolean(true)))
//     }

//     #[test]
//     fn separate_multiline_attributes() {
//         let data = r#"description = "Master key used for creating/decrypting cache token data keys"
//     enable_key_rotation = true
// }"#;
//         let (_, res) = separated_attributes(data).unwrap();
//         assert_eq!(res, vec![("description", AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))), ("enable_key_rotation", AttributeType::Boolean(true))])
//     }

//     #[test]
//     fn array_attributes() {
//         let data = r#"
// resource "aws_cloudwatch_metric_alarm" "discovery_diff-engine-queue-cloudwatch-alaram-messages-high" {
//   alarm_name = "discovery_cloudwatch-diff-engine-queue-cloudwatch-alarm-messages-high"

//   alarm_actions = ["${aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn}"]
// }        
// "#;
//         let block = TerraformBlockWithTwoIdentifiers { 
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_cloudwatch_metric_alarm"),
//             second_identifier: String::from("discovery_diff-engine-queue-cloudwatch-alaram-messages-high"),
//             attributes: vec![
//                 Attribute { key: String::from("alarm_name"), value: AttributeType::Str(String::from( "discovery_cloudwatch-diff-engine-queue-cloudwatch-alarm-messages-high")) },
//                 Attribute { key: String::from("alarm_actions"), value: AttributeType::Array(
//                     vec![AttributeType::Str(String::from("${aws_appautoscaling_policy.discovery_diff-engine-autoscaling-up.arn}"))]
//                 )}
//             ] 
//         };
//         let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
//         let (rest, result) = root(data).unwrap();
//         println!("rest: {}", rest);
//         assert_eq!(result, expected)
//     }

//     #[test]
//     fn terraform_block() {
//         let data = r#"
// # a line of comments to be ignored
// resource "aws_kms_key" "discovery_cache-master-key" {
//     description = "Master key used for creating/decrypting cache token data keys"
//     enable_key_rotation = true
// }
// "#;
//         let result = root(data);
//         let first_attr = Attribute {
//             key: String::from("description"),
//             value: AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))
//         };
//         let second_attr = Attribute {
//             key: String::from("enable_key_rotation"),
//             value: AttributeType::Boolean(true)
//         };
//         let block = TerraformBlockWithTwoIdentifiers {
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_kms_key"),
//             second_identifier: String::from("discovery_cache-master-key"),
//             attributes: vec![first_attr, second_attr]
//         };
//         let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
//         assert_eq!(result, Ok(("\n", expected)))
//     }

//     #[test]
//     fn terraform_parse_nested_block() {
//         let data = r#"
// resource "aws_cloudwatch_log_metric_filter" "discovery_diff-tagging-failed-event-error" {
//   name           = "diff_tagging_failed_event"

//   metric_transformation {
//     name      = "diff_tagging_failed_event"
//     namespace = "diff_tagging_log_metrics"
//     value     = "1"
//   }
// }
// "#;
//         let block = TerraformBlockWithTwoIdentifiers { 
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_cloudwatch_log_metric_filter"),
//             second_identifier: String::from("discovery_diff-tagging-failed-event-error"),
//             attributes: vec![
//                 Attribute { key: String::from("name"), value: AttributeType::Str(String::from("diff_tagging_failed_event")) },
//                 Attribute { key: String::from("metric_transformation"), value: AttributeType::Block(
//                     vec![
//                         (String::from("name"), AttributeType::Str(String::from("diff_tagging_failed_event"))), 
//                         (String::from("namespace"), AttributeType::Str(String::from("diff_tagging_log_metrics"))), (String::from("value"), AttributeType::Str(String::from("1")))
//                     ]
//                 )}
//             ] 
//         };
//         let expected = vec![TerraformBlock::WithTwoIdentifiers(block)];
//         let (_, result) = root(data).unwrap();
//         assert_eq!(result, expected)
//     }

//     #[test]
//     fn terraform_multiple_blocks() {
//         let data = r#"
// resource "aws_kms_key" "discovery_cache-master-key" {
//     description = "Master key used for creating/decrypting cache token data keys"
//     enable_key_rotation = true
// }

// resource "aws_lambda_event_source_mapping" "discovery_publisher-lambda-sqs-mapping" {
//   batch_size        = "1"
//   enabled           = true
// }
// "#;
//         let result = root(data);
//         let first_attr = Attribute {
//             key: String::from("description"),
//             value: AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))
//         };
//         let second_attr = Attribute {
//             key: String::from("enable_key_rotation"),
//             value: AttributeType::Boolean(true)
//         };
//         let first_attr1 = Attribute {
//             key: String::from("batch_size"),
//             value: AttributeType::Str(String::from("1"))
//         };
//         let second_attr2 = Attribute {
//             key: String::from("enabled"),
//             value: AttributeType::Boolean(true)
//         };
//         let block = TerraformBlockWithTwoIdentifiers {
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_kms_key"),
//             second_identifier: String::from("discovery_cache-master-key"),
//             attributes: vec![first_attr, second_attr]
//         };
//         let block2 = TerraformBlockWithTwoIdentifiers {
//             block_type: String::from("resource"),
//             first_identifier: String::from("aws_lambda_event_source_mapping"),
//             second_identifier: String::from("discovery_publisher-lambda-sqs-mapping"),
//             attributes: vec![first_attr1, second_attr2]
//         };
//         let expected = vec![TerraformBlock::WithTwoIdentifiers(block), TerraformBlock::WithTwoIdentifiers(block2)];
//         assert_eq!(result, Ok(("\n", expected)))
//     }

//     #[test]
//     fn terraform_comments() {
//         let input = "# [ ] create test for terraform block\nresource \"aws_kms_key\" \"discovery_cache-master-key\" {}";

//         let (left, _) = comment_one_line(input).unwrap();
//         assert_eq!(left, "resource \"aws_kms_key\" \"discovery_cache-master-key\" {}")
//     }

//     #[test]
//     fn identifier() {
//         let (_, res) = valid_identifier("aws_kms_key").unwrap();
//         assert_eq!(res, "aws_kms_key")
//     }

//     #[test]
//     fn terraform_templated_attribute() {
//         let kv_2 = "endpoint             = \"${aws_sqs_queue.discovery_collector-queue.arn}\"";
        
//         let (_, result) = key_value(kv_2).unwrap();
//         println!("result: {:?}", result);
//         assert_eq!(result, ("endpoint", AttributeType::Str(String::from("${aws_sqs_queue.discovery_collector-queue.arn}"))))
//     }

    // #[test]
    // fn test_take_and_consume() {
    //     let kv_2 = "endpoint             = \"${aws_sqs_queue.discovery_collector-queue.arn}stuffafterthebrace\"";
        
    //     let (leftover, result) = take_and_consume(kv_2).unwrap();
    //     println!("leftover: {}, result: {:?}", leftover, result);
    //     assert_eq!(leftover, "")
    // }

    // #[test]
    // fn terraform_string() {
    //     let kv_2 = "endpoint             = \"aws_sqs_queue.discovery_collector-queue.arn\"";
        
    //     let (leftover, result) = string::<(&str, ErrorKind)>(kv_2).unwrap();
    //     println!("leftover: {}", leftover);
    //     assert_eq!(leftover, "endpoint             = \"${aws_sqs_queue.discovery_collector-queue.arn}\"")
    // }
}
