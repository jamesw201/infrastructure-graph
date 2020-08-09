# Configuring Relationships
Relationships can be found in almost every resource in a cloud template. `TerraformParser`(TODO: Give name to application) uses a yaml file to specify where these relationships can be found for each Provider (Ie. `aws_relationships.yaml`).

Most relationships can be specified by simply referencing a source and target attribute within the root object:  

## Basic relationship configuration

**Example Resource:** aws_lambda_event_source_mapping
```
{
      "type": "aws_lambda_event_source_mapping",
      "name": "discovery_diff-tagging_EvtMap_discovery_event-bus",
      "body": {
        "depends_on": [
          "aws_lambda_function.discovery_diff-tagging"
        ],
        "starting_position": "LATEST",
        "batch_size": "50.0",
        "event_source_arn": "arn:aws:kinesis:us-east-1:309983114184:stream/discovery_event-bus",
        "function_name": "discovery_diff-tagging"
      }
}
```

**Relationship config:**
```
aws_lambda_event_source_mapping:
    source: event_source_arn
    target: function_name
    label: ""
```
In this case the relationship is [event_source_arn -> function_name].  
* Where `_arn` is found, the resource name will be extracted from the value.

**Extracts relationship**
```
[
    {
        source: "arn:aws:kinesis:us-east-1:309983114184:stream/discovery_event-bus",
        target: "discovery_diff-tagging",
        label: ""
    }
]
```

## Complex relationships configuration

Some resources contain multiple relationships nested deeper in their structure.

An `aws_iam_role_policy` contains an array of `Statements`, each statement can contain multiple `Resources`. The relationship config needs to extract a Relationship for each of these Resources. It also needs to keep track of the list of items in `Action` for the Relationship label. 

**Example Resource:** aws_iam_role_policy
```
{
      "type": "aws_iam_role_policy",
      "name": "discovery_consistency-checker_role_policy",
      "body": {
        "depends_on": [
          "aws_iam_role.discovery_consistency-checker_role"
        ],
        "policy": {
          "Statement": [
            {
              "Action": [
                "logs:CreateLogStream",
                "logs:CreateLogGroup",
                "logs:PutLogEvents"
              ],
              "Effect": "Allow",
              "Resource": [
                "arn:aws:logs:*:*:log-group:/aws/lambda/*discovery_consistency-checker*"
              ]
            },
            {
              "Action": [
                "dynamodb:GetItem",
                "dynamodb:Query"
              ],
              "Effect": "Allow",
              "Resource": [
                "arn:aws:dynamodb:us-east-1:309983114184:table/authorisation_token/*",
                "arn:aws:dynamodb:us-east-1:309983114184:table/authorisation_token",
                "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key/*",
                "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key"
              ]
            }
        ],
        "Version": "2012-10-17"
    },
    "role": "aws_iam_role.discovery_consistency-checker_role.id",
    "name": "discovery_consistency-checker_role_policy"
}
```

**Relationship config:**
```
aws_iam_role_policy:
    source: role
    policy.Statement
      - target: Resource
      - label: Action
```

**Extracts relationships**
```
[
    {
        source: aws_iam_role.discovery_consistency-checker_role,
        target: "arn:aws:dynamodb:us-east-1:309983114184:table/authorisation_token/*",
        label: "dynamodb:GetItem,dynamodb:Query"
    },
    {
        source: aws_iam_role.discovery_consistency-checker_role,
        target: "arn:aws:dynamodb:us-east-1:309983114184:table/authorisation_token",
        label: "dynamodb:GetItem,dynamodb:Query"
    },
    {
        source: aws_iam_role.discovery_consistency-checker_role,
        target: "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key/*",
        label: "dynamodb:GetItem,dynamodb:Query"
    },
    {
        source: aws_iam_role.discovery_consistency-checker_role,
        target: "arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key",
        label: "dynamodb:GetItem,dynamodb:Query"
    }
]
```
This config will generate a list of Relationships because of the array syntax: `Statement[]`. Array syntax can only exist in target, array syntax in the source will result in an RelationshipConfigurationError.  

TODO: Deduplication will have to take place to reduce wildcard relationship false positives.
```
"arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key"
and
"arn:aws:dynamodb:us-east-1:309983114184:table/authentication_key/*"

should result in a single relationship.
```

## Relationship aggregation rules...
TODO: decide if aggregation rules belong in the yaml syntax.