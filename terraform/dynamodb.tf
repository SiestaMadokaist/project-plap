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

resource "aws_dynamodb_table" "users" {
  name         = "${var.stage}-users"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "username"

  attribute {
    name = "username"
    type = "S"
  }

  attribute {
    name = "address"
    type = "S"
  }

  # find() / login() look a user up by wallet address and read the whole record
  # back, so the projection has to carry every UserItem field (user.rs).
  global_secondary_index {
    name            = "address"
    hash_key        = "address"
    projection_type = "ALL"
  }

  tags = {
    Service = var.service_name
    Stage   = var.stage
  }
}

# HotReloadRepository (domain/hot_reload.rs, infras/repos/dynamo/hotreload.rs). One item per
# (username, context) pair - `context` is "bill" | "launch" (HotReloadCfg's serde tag), so a
# user can hold a BillOptimization record and a LaunchConfig record at once.
# bill_optimization/launch_config: GetItem on the full key. get: Query on `username` alone,
# returning every context that user has.
resource "aws_dynamodb_table" "hot_reloads" {
  name         = "${var.stage}-hot_reloads"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "username"
  range_key    = "context"

  attribute {
    name = "username"
    type = "S"
  }

  attribute {
    name = "context"
    type = "S"
  }

  tags = {
    Service = var.service_name
    Stage   = var.stage
  }
}

resource "aws_dynamodb_table" "agent_commands" {
  name         = "${var.stage}-agent_commands"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "action_id"

  attribute {
    name = "action_id"
    type = "S"
  }

  attribute {
    name = "stage"
    type = "S"
  }

  attribute {
    name = "priority"
    type = "N"
  }

  # Query commands by stage (e.g. in_progress), ordered by priority within that stage
  global_secondary_index {
    name            = "stage-priority"
    hash_key        = "stage"
    range_key       = "priority"
    projection_type = "ALL"
  }

  ttl {
    attribute_name = "expire_at"
    enabled        = true
  }

  tags = {
    Service = var.service_name
    Stage   = var.stage
  }
}
