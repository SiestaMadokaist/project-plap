resource "aws_dynamodb_table" "translations" {
  name         = "${var.stage}-translations"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "novel_id"
  range_key    = "chapter_id"

  attribute {
    name = "novel_id"
    type = "S"
  }

  attribute {
    name = "chapter_id"
    type = "N"
  }

  attribute {
    name = "created_at"
    type = "N"
  }

  attribute {
    name = "status"
    type = "S"
  }

  # Query chapters of a novel sorted by time
  local_secondary_index {
    name            = "novel-by-created-at"
    range_key       = "created_at"
    projection_type = "ALL"
  }

  # Query all chapters across novels by processing status, oldest first
  global_secondary_index {
    name            = "status-by-created-at"
    hash_key        = "status"
    range_key       = "created_at"
    projection_type = "ALL"
  }

  tags = {
    Service = var.service_name
    Stage   = var.stage
  }
}
