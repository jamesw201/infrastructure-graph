# ==============================================
# Imported from Terraform files

# ----------------------------------------------------------
# Master key used for encrypting/decrypting discovery collector
# cache tokens
resource "aws_kms_key" "discovery_cache-master-key" {
    description = "Master key used for creating/decrypting cache token data keys"
    enable_key_rotation = true
}
