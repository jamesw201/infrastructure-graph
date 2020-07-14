# How to query this data structure with CloudCustodian "like" syntax?

The challenge is to filter nested collections where more than one attribute value needs to be filtered on.

With data:
```
{
  policy: {
    Statement: [
      {
        "Action": [
          "dynamodb:GetItem",
          "dynamodb:UpdateItem"
        ],
        "Effect": "Allow",
        "Resource": [
          "arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config/*",
          "arn:aws:dynamodb:us-east-1:309983114184:table/discovery_tenant-config"
        ]
      }
    ]
  }
}
```

```
filters:

    OLD Syntax:
    - key: 'policy.Statement[].Resource'
        op: contains
        value: /arn:aws:dynamodb:{region}:{accountId};table/*/g
    - key: 'policy.Statement[].Action'
        op: contains
        value: /arn:aws:dynamodb:{region}:{accountId};table/*/g


    NEW Syntax:
    - key: 'policy.Statement[]'
      op: filter
      value:
        - key: Action
          op: contains
          value: 'dynamodb:UpdateItem'
        - key: Resource
          op: contains
          value: /arn:aws:dynamodb:{region}:{accountId};table/*/g
        - key: Effect
          op: equals
          value: 'Allow'
```

The `filter` op should only be available if the key ends with the array `[]` token.
