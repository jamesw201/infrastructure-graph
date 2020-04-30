extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{escaped, is_not, tag, take, take_while, take_until},
  character::complete::{alphanumeric1, char, one_of, multispace0, newline, not_line_ending, line_ending, space0},
  combinator::{map, opt, value},
  error::{ParseError},
  multi::{many0, separated_list0},
  number::complete::double,
  sequence::{delimited, preceded, separated_pair, terminated},
  IResult
};

use std::collections::HashMap;
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum AttributeType {
    Str(String),
    TemplateString(String),
    Boolean(bool),
    Num(f64),
    Array(Vec<AttributeType>),
    Block(HashMap<String, AttributeType>),
    Json()
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


#[allow(dead_code)]
fn take_and_consume(i: &str) -> IResult<&str, &str> {
    let (res, _) = take_until("}")(i)?;
    take(1usize)(res)
}

#[allow(dead_code)]
fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
  let parse_true = value(true, tag("true"));
  let parse_false = value(false, tag("false"));

  alt((parse_true, parse_false))(input)
}

fn parse_single_line_str(i: &str) -> IResult<&str, &str> {
    preceded(space0, escaped(is_not("\""), '\\', one_of(r#"\"#)))(i)
}

// fn parse_str(i: &str) -> IResult<&str, &str> {
//     preceded(space0, escaped(is_not("\\\n"), '\\', one_of(r#"\"#)))(i)
// }

#[allow(dead_code)]
fn comment_one_line(i: &str) -> IResult<&str, ()> {
    let (rest, _) = alt((tag("//"), tag("#")))(i)?;
    let (rest, _) = opt(not_line_ending)(rest)?;
    let (rest, _) = line_ending(rest)?;

    Ok((rest, ()))
}

fn valid_identifier_char(c: char) -> bool {
  c.is_alphanumeric() || c == '_'
}

fn valid_identifier(i: &str) -> IResult<&str, &str> {
    take_while(valid_identifier_char)(i)
}

#[allow(dead_code)]
fn key_value(i: &str) -> IResult<&str, (&str, AttributeType)> {
    separated_pair(preceded(space0, valid_identifier), preceded(space0, char('=')), block_value)(i)
}

fn block_value(i: &str) -> IResult<&str, AttributeType> {
  preceded(
    space0,
    alt((
      map(boolean, AttributeType::Boolean),
      map(double, AttributeType::Num),
      map(escaped_string, |s| AttributeType::Str(String::from(s))),
    )),
  )(i)
}

fn escaped_string(i: &str) -> IResult<&str, &str> {
    preceded(char('\"'), terminated(parse_single_line_str, char('\"')))(i)
}

fn separated_attributes(i: &str) -> IResult<&str, Vec<(&str, AttributeType)>> {
    separated_list0(preceded(space0, newline), key_value)(i)
}

fn build_tf_block(identifiers: Vec<&str>, attributes: Vec<(String, AttributeType)>) -> TerraformBlock {
    match identifiers.len() {
        1 => {
            TerraformBlock::NoIdentifiers(
                TerraformBlockWithNoIdentifiers {
                    block_type: identifiers[0].to_string(),
                    attributes: attributes.into_iter().map(|(key, value)| Attribute{key, value}).collect()
                }
            )
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
        _ => panic!("encountered terraform block with too many identifiers")
    }
}

fn tf_block(i: &str) -> IResult<&str, TerraformBlock> {
    let (rest, identifiers) = 
        delimited(
            multispace0,
            many0(
                preceded(space0, alt((alphanumeric1, escaped_string)))
            ),
            opt(space0)
        )(i)?;
    println!("identifiers: {:?}", identifiers);

    let (rest, attributes): (&str, Vec<(String, AttributeType)>) = 
        preceded(
            char('{'),
            preceded(multispace0,
                terminated(
                    map(
                        separated_attributes, 
                            |tuple_vec| {
                                tuple_vec.into_iter().map(|(k, v)| (String::from(k), v)).collect()
                            }
                        ),
                    preceded(multispace0, char('}')),
                )
            ),
        )(rest)?;

    println!("attributes: {:?}", attributes);

    let block = build_tf_block(identifiers, attributes);
    Ok((rest, block))
}

/// the root element
#[allow(dead_code)]
fn root(i: &str) -> IResult<&str, Vec<TerraformBlock>> {
    many0(
        preceded(multispace0, tf_block)
    )(i)
}

// TODO: 
// [√] parse multiple resources separated by blank lines
// [ ] parse multiple resources separated by blank lines and comment lines
// [ ] parse nested blocks
// [ ] parse nested json blocks
// [ ] build relationships from templated attribute values
// [ ] build relationships json values

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_value_escaped_string() {
        let data = r#"description = "Master key used for creating/decrypting cache token data keys""#;
        let (_, res) = key_value(data).unwrap();
        println!("res: {:?}", res);
        assert_eq!(res, ("description", AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))))
    }

    #[test]
    fn key_value_snake_case_identifier() {
        let data = r#"enable_key_rotation = true"#;
        let (_, res) = key_value(data).unwrap();
        assert_eq!(res, ("enable_key_rotation", AttributeType::Boolean(true)))
    }

    #[test]
    fn separate_multiline_attributes() {
        let data = r#"description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}"#;
        let (_, res) = separated_attributes(data).unwrap();
        assert_eq!(res, vec![("description", AttributeType::Str(String::from("Master key used for creating/decrypting cache token data keys"))), ("enable_key_rotation", AttributeType::Boolean(true))])
    }

    #[test]
    fn terraform_block() {
        let data = r#"
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
    fn terraform_multiple_blocks() {
        let data = r#"
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}

resource "aws_lambda_event_source_mapping" "discovery_publisher-lambda-sqs-mapping" {
  batch_size        = "1"
  enabled           = true
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
        let first_attr1 = Attribute {
            key: String::from("batch_size"),
            value: AttributeType::Str(String::from("1"))
        };
        let second_attr2 = Attribute {
            key: String::from("enabled"),
            value: AttributeType::Boolean(true)
        };
        let block = TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_kms_key"),
            second_identifier: String::from("discovery_cache-master-key"),
            attributes: vec![first_attr, second_attr]
        };
        let block2 = TerraformBlockWithTwoIdentifiers {
            block_type: String::from("resource"),
            first_identifier: String::from("aws_lambda_event_source_mapping"),
            second_identifier: String::from("discovery_publisher-lambda-sqs-mapping"),
            attributes: vec![first_attr1, second_attr2]
        };
        let expected = vec![TerraformBlock::WithTwoIdentifiers(block), TerraformBlock::WithTwoIdentifiers(block2)];
        assert_eq!(result, Ok(("\n", expected)))
    }

    #[test]
    fn terraform_comments() {
        let input = "# [ ] create test for terraform block\nresource \"aws_kms_key\" \"discovery_cache-master-key\" {}";

        let (left, _) = comment_one_line(input).unwrap();
        assert_eq!(left, "resource \"aws_kms_key\" \"discovery_cache-master-key\" {}")
    }

    #[test]
    fn identifier() {
        let (_, res) = valid_identifier("aws_kms_key").unwrap();
        assert_eq!(res, "aws_kms_key")
    }

    #[test]
    fn terraform_templated_attribute() {
        let kv_2 = "endpoint             = \"${aws_sqs_queue.discovery_collector-queue.arn}\"";
        
        let (_, result) = key_value(kv_2).unwrap();
        println!("result: {:?}", result);
        assert_eq!(result, ("endpoint", AttributeType::Str(String::from("${aws_sqs_queue.discovery_collector-queue.arn}"))))
    }

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


// pub fn run() -> Result<(), Box<dyn Error>> {
//   let data = "  { \"a\"\t: 42,
//   \"b\": [ \"x\", \"y\", 12 ] ,
//   \"c\": { \"hello\" : \"world\"
//   }
//   } ";

//   println!("will try to parse valid JSON data:\n\n**********\n{}\n**********\n", data);

//   // this will print:
//   // Ok(
//   //     (
//   //         "",
//   //         Object(
//   //             {
//   //                 "b": Array(
//   //                     [
//   //                         Str(
//   //                             "x",
//   //                         ),
//   //                         Str(
//   //                             "y",
//   //                         ),
//   //                         Num(
//   //                             12.0,
//   //                         ),
//   //                     ],
//   //                 ),
//   //                 "c": Object(
//   //                     {
//   //                         "hello": Str(
//   //                             "world",
//   //                         ),
//   //                     },
//   //                 ),
//   //                 "a": Num(
//   //                     42.0,
//   //                 ),
//   //             },
//   //         ),
//   //     ),
//   // )
//   println!("parsing a valid file:\n{:#?}\n", root::<(&str, ErrorKind)>(data));

//   let data = "  { \"a\"\t: 42,
//   \"b\": [ \"x\", \"y\", 12 ] ,
//   \"c\": { 1\"hello\" : \"world\"
//   }
//   } ";

//   println!("will try to parse invalid JSON data:\n\n**********\n{}\n**********\n", data);

//   // here we use `(Input, ErrorKind)` as error type, which is used by default
//   // if you don't space0ecify it. It contains the position of the error and some
//   // info on which parser encountered it.
//   // It is fast and small, but does not provide much context.
//   //
//   // This will print:
//   // basic errors - `root::<(&str, ErrorKind)>(data)`:
//   // Err(
//   //   Failure(
//   //       (
//   //           "1\"hello\" : \"world\"\n  }\n  } ",
//   //           Char,
//   //       ),
//   //   ),
//   // )
//   println!(
//     "basic errors - `root::<(&str, ErrorKind)>(data)`:\n{:#?}\n",
//     root::<(&str, ErrorKind)>(data)
//   );

//   // nom also provides `the `VerboseError<Input>` type, which will generate a sort
//   // of backtrace of the path through the parser, accumulating info on input positions
//   // and affected parsers.
//   //
//   // This will print:
//   //
//   // parsed verbose: Err(
//   //   Failure(
//   //       VerboseError {
//   //           errors: [
//   //               (
//   //                   "1\"hello\" : \"world\"\n  }\n  } ",
//   //                   Char(
//   //                       '}',
//   //                   ),
//   //               ),
//   //               (
//   //                   "{ 1\"hello\" : \"world\"\n  }\n  } ",
//   //                   Context(
//   //                       "map",
//   //                   ),
//   //               ),
//   //               (
//   //                   "{ \"a\"\t: 42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } ",
//   //                   Context(
//   //                       "map",
//   //                   ),
//   //               ),
//   //           ],
//   //       },
//   //   ),
//   // )
//   println!("parsed verbose: {:#?}", root::<VerboseError<&str>>(data));

//   match root::<VerboseError<&str>>(data) {
//     Err(Err::Error(e)) | Err(Err::Failure(e)) => {
//       // here we use the `convert_error` function, to transform a `VerboseError<&str>`
//       // into a printable trace.
//       //
//       // This will print:
//       // verbose errors - `root::<VerboseError>(data)`:
//       // 0: at line 2:
//       //   "c": { 1"hello" : "world"
//       //          ^
//       // expected '}', found 1
//       //
//       // 1: at line 2, in map:
//       //   "c": { 1"hello" : "world"
//       //        ^
//       //
//       // 2: at line 0, in map:
//       //   { "a" : 42,
//       //   ^
//       println!("verbose errors - `root::<VerboseError>(data)`:\n{}", convert_error(data, e));
//     }
//     _ => Ok(())
//   }
// }
