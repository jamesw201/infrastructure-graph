extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{ escaped, tag, is_not, take_till, take_while },
  character::complete::{ char, multispace0, one_of, space0 },
  combinator::{cut, map, opt, value},
  multi::separated_list0,
  number::complete::double,
  sequence::{delimited, preceded, separated_pair, terminated},
  IResult,
};
use std::str;

use crate::structs::json::JsonValue;

fn parse_single_line_str(i: &str) -> IResult<&str, String> {
    let (rest, result) = preceded(space0, escaped(is_not("\""), '\\', one_of(r#"\"#)))(i)?;
    Ok((rest, String::from(result)))
}

fn boolean(input: &str) -> IResult<&str, bool> {
  let parse_true = value(true, tag("true"));
  let parse_false = value(false, tag("false"));

  alt((parse_true, parse_false))(input)
}

fn null(input: &str) -> IResult<&str, String> {
  value(String::from("null"), tag("null"))(input)
}

fn string(i: &str) -> IResult<&str, String> {
    alt((
        preceded(char('\"'), terminated(parse_single_line_str, char('\"'))),
        value(String::from(""), tag("\"\"")),
    ))(i)
}

fn array(i: &str) -> IResult<&str, Vec<JsonValue>> {
    preceded(
      preceded(multispace0, char('[')),
      terminated(
        separated_list0(preceded(space0, char(',')), json_value),
        preceded(multispace0, char(']')),
      ),
    )(i)
}

fn valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '/'
}

fn valid_identifier(i: &str) -> IResult<&str, &str> {
    take_while(valid_identifier_char)(i)
}

fn escaped_l_string(i: &str) -> IResult<&str, &str> {
    let (rest, result) = preceded(
        multispace0,
        alt((
            delimited(tag("\\\""), valid_identifier, tag("\\\"")),
            delimited(tag("\\"), valid_identifier, tag("\\")),
            delimited(tag("\""), valid_identifier, tag("\"")),
        ))
    )(i)?;
    Ok((rest, result))
}

fn valid_r_string(i: &str) -> IResult<&str, &str> {
    take_till(|c: char| (
        !c.is_alphanumeric() && 
        c != ':' &&
        c != '.' &&
        c != '_' &&
        c != '$' &&
        c != '{' &&
        c != '}' &&
        c != ' ' &&
        c != '/' &&
        c != '#' &&
        c != '*' &&
        c != '-'
    ))(i)
}

fn escaped_r_string(i: &str) -> IResult<&str, &str> {
    let (rest, result) = preceded(
        multispace0,
        terminated(
            alt((
                delimited(tag("\\\""), valid_r_string, tag("\\\"")),
                delimited(tag("\\"), valid_r_string, tag("\\")),
                delimited(tag("\""), valid_r_string, tag("\"")),
            )),
            multispace0
        )
    )(i)?;
    Ok((rest, result))
}

fn key_value(i: &str) -> IResult<&str, (&str, JsonValue)> {
    separated_pair(
        escaped_l_string,
        cut(preceded(space0, char(':'))),
        json_value,
    )(i)
}

fn hash(i: &str) -> IResult<&str, Vec<(String, JsonValue)>> {
    preceded(
        preceded(multispace0, char('{')),
        terminated(
            map(
            separated_list0(preceded(space0, char(',')), key_value),
            |tuple_vec| {
                tuple_vec
                .into_iter()
                .map(|(k, v)| (String::from(k), v))
                .collect()
            },
            ),                                                                                                                                      
            preceded(multispace0, char('}')),
        )   ,
        )(i)
}

/// here, we apply the space0 parser before trying to parse a value
fn json_value(i: &str) -> IResult<&str, JsonValue> {
  preceded(
    multispace0,
    alt((
      map(hash, JsonValue::Object),
      map(array, JsonValue::Array),
      map(string, |s| JsonValue::Str(String::from(s))),
      map(escaped_r_string, |s| JsonValue::Str(String::from(s))),
      map(double, JsonValue::Num),
      map(boolean, JsonValue::Boolean),
      map(null, JsonValue::Null),
    )),
  )(i)
}

/// the root element of a JSON parser is either an object or an array
pub fn parse_json(i: &str) -> IResult<&str, JsonValue> {
    delimited(
        space0,
        alt((map(hash, JsonValue::Object), map(array, JsonValue::Array))),
        opt(space0),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_escaped_string() {
        let data = "\\\"deadLetterTargetArn\\\"";
        assert_eq!(escaped_l_string(data), Ok(("", "deadLetterTargetArn")));
    }

    #[test]
    fn parse_r_blarp_string() {
        let data = "arn:blarp\n";
        assert_eq!(valid_r_string(data), Ok(("\n", "arn:blarp")));
    }

    #[test]
    fn parse_serialised_json() {
        let data = "{\\\"deadLetterTargetArn\\\":\\\"arn:blarp\\\"}";
        let (_, result) = parse_json(data).unwrap();
        let expected = JsonValue::Object(vec![(String::from("deadLetterTargetArn"), JsonValue::Str(String::from("arn:blarp")))]);
        assert_eq!(result, expected)
    }

    #[test]
    fn parse_serialised_json_with_rest() {
        let data = "{\\\"deadLetterTargetArn\\\":\\\"${aws_sqs_queue.discovery_collector-deadletter-queue.arn}\\\",\\\"maxReceiveCount\\\":2}\n    tags {\n      Environment        = \"sandbox1\"\n    }\n}\n";
        let (_, result) = parse_json(data).unwrap();
        let expected = JsonValue::Object(vec![(String::from("deadLetterTargetArn"), JsonValue::Str(String::from("${aws_sqs_queue.discovery_collector-deadletter-queue.arn}"))), (String::from("maxReceiveCount"), JsonValue::Num(2.0))]);
        assert_eq!(result, expected)
    }

    #[test]
    fn parse_non_serialised_json() {
        let data = r#"
{
    "deadLetterTargetArn": "arn:blarp"
}
"#;
        let (_, result) = parse_json(data).unwrap();
        let expected = JsonValue::Object(vec![(String::from("deadLetterTargetArn"), JsonValue::Str(String::from("arn:blarp")))]);
        assert_eq!(result, expected)
    }

    #[test]
    fn null_entry() {
        let data = r#"
[
  {
    "entryPoint": null,
    "essential": true
  }
]
"#;
        let (_, result) = parse_json(data).unwrap();
        let expected = JsonValue::Array(vec![JsonValue::Object(vec![(String::from("entryPoint"), JsonValue::Null(String::from("null"))), (String::from("essential"), JsonValue::Boolean(true))])]);
        assert_eq!(result, expected)
    }

    // #[test]
    // fn parse_strings() {
    //     let string_with_underscores = "under_score";
    //     let string_with_hyphens = "hyphen-hyphen";
    //     let string_with_full_stops = "full.stops";

    //     assert_eq!(parse_single_line_str(string_with_underscores), Ok(("", "under_score")));
    //     assert_eq!(parse_single_line_str(string_with_hyphens), Ok(("", "hyphen-hyphen")));
    //     assert_eq!(parse_single_line_str(string_with_full_stops), Ok(("", "full.stops")));
    // }

    // #[test]
    // fn empty_string_values() {
    //     assert_eq!(string("\"\""), Ok(("", "")))
    // }

    #[test]
    fn parse_hash() {
        let data = r#"
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Principal": {
        "Service": "vpc-flow-logs.amazonaws.com"
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
"#;
        let expected = JsonValue::Object(
            vec![
                (String::from("Version"), JsonValue::Str(String::from("2012-10-17"))),
                (String::from("Statement"), JsonValue::Array(
                    vec![
                        JsonValue::Object(
                            vec![
                                (String::from("Sid"), JsonValue::Str(String::from(""))), 
                                (String::from("Effect"), JsonValue::Str(String::from("Allow"))),
                                (String::from("Principal"), JsonValue::Object(
                                    vec![
                                        (String::from("Service"), JsonValue::Str(String::from("vpc-flow-logs.amazonaws.com")))
                                    ],
                                )),
                                (String::from("Action"), JsonValue::Str(String::from("sts:AssumeRole")))
                            ]
                        )
                    ])
                ),
            ]
        );

        let (_, result) = parse_json(data).unwrap();
        assert_eq!(result, expected)
    }
}
