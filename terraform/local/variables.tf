# only what ../dynamodb.tf references
# `stage` deliberately differs from ../variables.tf ("production"): local tables are
# created as development-*, the production stack keeps production-*.
variable "stage" {
  default = "development"
}

# keep in sync with ../variables.tf
variable "service_name" {
  default = "rust-api"
}
