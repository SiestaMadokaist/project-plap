# Creates the DynamoDB tables in the local dynamodb-local container (see `make dynamo`)
# by reusing ../dynamodb.tf. Nothing else from the parent module is applied here.
terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  # state stays on disk, so this can never touch the production S3 state
  backend "local" {}
}

provider "aws" {
  region                      = "ap-southeast-1"
  access_key                  = "local"
  secret_key                  = "local"
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true

  endpoints {
    dynamodb = "http://localhost:40000"
  }
}
