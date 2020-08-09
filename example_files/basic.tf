# ==============================================
# Imported from Terraform files
#
# ----------------------------------------------------------
# Master key used for encrypting/decrypting discovery collector
# cache tokens
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}

# ==============================================
# aws_s3_bucket_object : ../service-discovery//infrastructure/aws_s3_bucket_object.yaml

# ----------------------------------------------
resource "aws_s3_bucket_object" "discovery_cpsc-vmware-config" {
    bucket               = "acp-platform-s-discovery-sandbox1"
    source               = "/Users/james.n.wilson/code/work/repos/cd-pipeline/../service-discovery//infrastructure/default-config/cpsc-vmware-config.json"
    etag                 = "${md5(file("/Users/james.n.wilson/code/work/repos/cd-pipeline/../service-discovery//infrastructure/default-config/cpsc-vmware-config.json"))}"
    key                  = "default-config/cpsc-vmware-config.json"
}
