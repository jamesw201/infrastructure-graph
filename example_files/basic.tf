# ==============================================
# Imported from Terraform files
#
# ----------------------------------------------------------
# Master key used for encrypting/decrypting discovery collector
# cache tokens
resource "aws_lambda_function" "discovery_api" {
    function_name        = "discovery_api"
    depends_on           = [ "aws_iam_role.discovery_api_role" ]
    role                 = "${aws_iam_role.discovery_api_role.arn}"
    filename             = "/tmp/nodejs_lambda_stub.zip"
    tags {
      Environment        = "sandbox1"
      Component          = "discovery"
      Name               = "discovery_api"
      CreatedBy          = "Platform.S create-infra.py"
    }
    handler              = "lib/index.handler"
    timeout              = 30
    lifecycle {
      ignore_changes     = ["handler", "environment"]
    }

    vpc_config {
      subnet_ids         = ["${aws_subnet.discovery_private-subnet-az-a.id}", "${aws_subnet.discovery_private-subnet-az-b.id}"]
      security_group_ids = ["${aws_security_group.discovery_cache-security-group1.id}"]
    }

    memory_size          = 768
    runtime              = "nodejs8.10"
    description          = "Discovery API"
}