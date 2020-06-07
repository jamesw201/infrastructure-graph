# Cloud Template Parser
This project will parse deployment templates (Terraform, Cloudformation) to generate a graph which can be used for a number of purposes:
- security policy evaluation (1000s in seconds)
- RDF conversion for graph database storage

Creating this project in Rust will hopefully provide optimal execution times.

## Design
#### CloudTemplateParser -> (nodes)
  - Reads in (Terraform, Cloudformation) templates -- might be replaced by separate FileReader Entity at some point.
  - Uses nom to create struct_tree of resources.

#### EdgeFinder(struct_tree) -> (nodes, edges)
  - Reads the tree, finding the relationships between the nodes.

  - ? What would be the fastest representation of resources to visit and build edges from..

#### PolicyEngine(policies)
  - Policies will be in a terse format that allows for thousands of policies to be read in one file.

#### RuleEvaluator
  - Same as before.


# Notes
- Build it initially as a cli app with local policies (a bit like cloud-custodian but offline).


# Tasks
[√] build basic tests for the CloudTemplateParser construct
[√] build skeleton entities for the CloudTemplateParser construct
[√] read in files
[√] translate our first template resource into a struct

[√] parse multiple resources separated by blank lines
[√] parse multiple resources separated by blank lines and comment lines
[√] parse nested blocks
[√] parse arrays
[√] parse nested json blocks
[√] parse serialised json blocks
[√] handle inline blocks:
```request_templates = { "application/json" = "{ \"statusCode\": 200 }" }```

[√] parse templated strings ```"${value.here}"```

[x] handle built-in functions:
```etag = "${md5(file("default-config/cpsc-vmware-config.json"))}"```

[ ] parse whole files from cli

[ ] create json transform from AST

[ ] build relationships from templated attribute values
[ ] build relationships from json values

[ ] create react diagrams from the output of the AST

[ ] handle templated expressions explicitely
[ ] refactor the terraform/json structs to bring them into line with each other

[ ] create security policies which run against the AST

[ ] create rdf transform from AST

[ ] show security violations on the react FE, overlays will highlight the offending resources
and show a code snippet of the offending template lines

[ ] deploy to lambda function
[ ] query the AST
[ ] create SRE pipeline including notebooks and Slack alarm posts

[ ] use machine learning to build intuitions based on the AST and supporting data
[ ] show graph changes over time in React app
[ ] create VR representaion of graph
