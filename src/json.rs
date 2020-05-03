extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{escaped, tag, is_not},
  character::complete::{char, multispace0, one_of, space0},
  combinator::{cut, map, opt, value},
  multi::separated_list0,
  number::complete::double,
  sequence::{delimited, preceded, separated_pair, terminated},
  IResult,
};
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(Vec<(String, JsonValue)>),
}

fn parse_single_line_str(i: &str) -> IResult<&str, &str> {
    preceded(space0, escaped(is_not("\""), '\\', one_of(r#"\"#)))(i)
}

fn boolean(input: &str) -> IResult<&str, bool> {
  let parse_true = value(true, tag("true"));
  let parse_false = value(false, tag("false"));

  alt((parse_true, parse_false))(input)
}

fn string(i: &str) -> IResult<&str, &str> {
    alt((
        preceded(char('\"'), terminated(parse_single_line_str, char('\"'))),
        value("", tag("\"\""))
    ))(i)
}

fn array(i: &str) -> IResult<&str, Vec<JsonValue>> {
    preceded(
      preceded(multispace0, char('[')),
      cut(terminated(
        separated_list0(preceded(space0, char(',')), json_value),
        preceded(multispace0, char(']')),
      )),
    )(i)
}

fn key_value(i: &str) -> IResult<&str, (&str, JsonValue)> {
  separated_pair(
    preceded(multispace0, string),
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
      map(double, JsonValue::Num),
      map(boolean, JsonValue::Boolean),
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
    fn parse_strings() {
        let string_with_underscores = "under_score";
        let string_with_hyphens = "hyphen-hyphen";
        let string_with_full_stops = "full.stops";

        assert_eq!(parse_single_line_str(string_with_underscores), Ok(("", "under_score")));
        assert_eq!(parse_single_line_str(string_with_hyphens), Ok(("", "hyphen-hyphen")));
        assert_eq!(parse_single_line_str(string_with_full_stops), Ok(("", "full.stops")));
    }

    #[test]
    fn empty_string_values() {
        assert_eq!(string("\"\""), Ok(("", "")))
    }

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
