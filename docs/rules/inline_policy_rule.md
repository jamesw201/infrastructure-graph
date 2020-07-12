###Inline Policy Rule
If you find an attribute with:
>    key: 'policy'
>    value: Json

Then we're looking at an inline policy.


**Terraform resource**
```
resource "aws_sqs_queue" "discovery_collector-queue" {
    visibility_timeout_seconds = 30
    kms_master_key_id    = "${aws_kms_key.discovery_master-key.arn}"
    name                 = "discovery_collector-queue"
    lifecycle {
      prevent_destroy    = true
    }
    policy               = "{ \"Version\": \"2012-10-17\", \"Id\": \"arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue/SNStoSQSPolicy\", \"Statement\": [ { \"Sid\": \"\", \"Effect\": \"Allow\", \"Principal\": { \"AWS\": \"*\" }, \"Action\": \"SQS:SendMessage\", \"Resource\": \"arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue\", \"Condition\": { \"ArnEquals\": { \"aws:SourceArn\": \"arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic\" } } } ] }"
    kms_data_key_reuse_period_seconds = 3600
    redrive_policy       = "{\"deadLetterTargetArn\":\"${aws_sqs_queue.discovery_collector-deadletter-queue.arn}\",\"maxReceiveCount\":2}"
    message_retention_seconds = 3600
    tags {
      Environment        = "sandbox1"
      Component          = "discovery"
      Name               = "discovery_collector-queue"
      CreatedBy          = "Platform.S create-infra.py"
    }
}
```

**Intermediate Syntax**
```
WithTwoIdentifiers(TerraformBlockWithTwoIdentifiers {   
    block_type: "resource", 
    first_identifier: "aws_sqs_queue", 
    second_identifier: "discovery_collector-queue",  
    attributes: [
        Attribute {
            key: "visibility_timeout_seconds",
            value: Num(30.0)
        },
        Attribute {
            key: "kms_master_key_id",
            value: TemplatedString(Variable("aws_kms_key.discovery_master-key.arn"))
        },
        Attribute {
            key: "name",
            value: Str("discovery_collector-queue")
        },
        Attribute {
            key: "lifecycle",
            value: Block([
                Attribute {
                    key: "prevent_destroy",
                    value: Boolean(true)
                }
            ])
        },
        Attribute {
            key: "policy",
            value: Json(
                Object([
                    ("Version", Str("2012-10-17")),
                    ("Id", Str("arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue/SNStoSQSPolicy")),
                    ("Statement", Array([
                        Object([
                            ("Sid", Str("")), 
                            ("Effect", Str("Allow")), 
                            ("Principal", Object([
                                ("AWS", Str("*"))
                            ])), 
                            ("Action", Str("SQS:SendMessage")),  
                            ("Resource", Str("arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue")), 
                            ("Condition", Object([
                                ("ArnEquals", Object([
                                    ("aws:SourceArn", Str("arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic"))
                                ]))
                            ]))
                        ])
                    ]))
                ])
            )
        },
        Attribute {
            key: "kms_data_key_reuse_period_seconds",
            value: Num(3600.0)
        },
        Attribute {
            key: "redrive_policy",
            value: Json(
                Object([
                    ("deadLetterTargetArn", Str("${aws_sqs_queue.discovery_collector-deadletter-queue.arn}")),
                    ("maxReceiveCount", Num(2.0))
                ])
            )
        },
        Attribute {
            key: "message_retention_seconds",
            value: Num(3600.0)
        },
        Attribute {
            key: "tags",
            value: Block([
                Attribute {
                    key: "Environment",
                    value: Str("sandbox1")
                },
                Attribute {
                    key: "Component",
                    value: Str("discovery")
                },
                Attribute {
                    key: "Name",
                    value: Str("discovery_collector-queue")
                },
                Attribute {
                    key: "CreatedBy",
                    value: Str("Platform.S create-infra.py")
                }
            ])
        }
    ]
})
```

**Json output**
```
{
    "type": "aws_sqs_queue",
    "name": "discovery_collector-queue",
    "body": {
      "visibility_timeout_seconds": "30.0",
      "kms_master_key_id": "aws_kms_key.discovery_master-key.arn",
      "name": "discovery_collector-queue",
      "lifecycle": {
        "prevent_destroy": "true"
      },
      "policy": {
        "Version": "2012-10-17",
        "Id": "arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue/SNStoSQSPolicy",
        "Statement": [
          {
            "Sid": "",
            "Effect": "Allow",
            "Principal": {
              "AWS": "*"
            },
            "Action": "SQS:SendMessage",
            "Resource": "arn:aws:sqs:us-east-1:309983114184:discovery_collector-queue",
            "Condition": {
              "ArnEquals": {
                "aws:SourceArn": "arn:aws:sns:us-east-1:309983114184:discovery_scheduled-discovery-topic"
              }
            }
          }
        ]
      },
      "kms_data_key_reuse_period_seconds": "3600.0",
      "redrive_policy": {
        "deadLetterTargetArn": "${aws_sqs_queue.discovery_collector-deadletter-queue.arn}",
        "maxReceiveCount": "2.0"
      },
      "message_retention_seconds": "3600.0",
      "tags": {
        "Environment": "sandbox1",
        "Component": "discovery",
        "Name": "discovery_collector-queue",
        "CreatedBy": "Platform.S create-infra.py"
      }
    }
  }
```