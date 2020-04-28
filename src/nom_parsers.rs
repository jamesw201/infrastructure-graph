extern crate nom;


#[derive(Clone, Default, Debug)]
pub struct Mount {
	pub device: std::string::String,
	pub mount_point: std::string::String,
	pub file_system_type: std::string::String,
	pub options: std::vec::Vec<std::string::String>,
}

#[derive(Clone, Default, Debug)]
pub struct Attribute {
    // key
    // value
}

#[derive(Clone, Default, Debug)]
pub struct TerraformResource {
    // type
    // name
    // attributes
}

#[derive(Clone, Debug, PartialEq)]
enum TerraformSyntax {
    // comment
    // resource
}

// TODO:
// [ ] create a TerraformResource parser for simple string key/value resource
// [ ] create a key/value parser
// [ ] create a block parser for key/value where the value is a block

pub(self) mod parsers {
	// use super::Mount;
	// use super::TerraformResource;
    use super::Attribute;

    #[allow(dead_code)]
	fn not_whitespace(i: &str) -> nom::IResult<&str, &str> {
		nom::bytes::complete::is_not(" \t")(i)
	}

    // fn attribute(i: &str) -> nom::IResult<&str, Attribute> {
	// 	nom::bytes::complete::is_not(" \t")(i)
	// }
	
	#[cfg(test)]
	mod json_tests {
		use super::*;
		
		#[test]
		fn test_not_whitespace() {
			assert_eq!(not_whitespace("abcd efg"), Ok((" efg", "abcd")));
			assert_eq!(not_whitespace("abcd\tefg"), Ok(("\tefg", "abcd")));
			assert_eq!(not_whitespace(" abcdefg"), Err(nom::Err::Error((" abcdefg", nom::error::ErrorKind::IsNot))));
		}

        // #[test]
        // #[ignore]
        // fn test_resource() {
        //     let input = "

        //         # Generating aws_sns_topic_subscription for aws_sns_topic
        //         # ----------------------------------------------
        //         resource \"aws_sns_topic_subscription\" \"discovery_scheduled-discovery-topic_Sub_sqs_discovery_collector-queue\" {
        //             protocol             = \"sqs\"
        //             endpoint             = \"${aws_sqs_queue.discovery_collector-queue.arn}\"
        //             topic_arn            = \"${aws_sns_topic.discovery_scheduled-discovery-topic.arn}\"
        //         }

        //     ";
        //     assert_eq!(root(input), Ok(("", vec![TerraformResource]));
        // }
	}
}
