locals {
  dist_dir = "${path.module}/../dist"
}

resource "aws_lambda_function" "api" {
  function_name    = "${var.service_name}-${var.stage}-api"
  role             = aws_iam_role.lambda.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 10
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/api.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/api.zip")

  environment {
    variables = {
      ENV_PATH     = ".env.${var.stage}"
      RUST_LOG     = "info"
      DYNAMO_TABLE = aws_dynamodb_table.translations.name
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.basic_execution,
    aws_cloudwatch_log_group.api,
  ]
}

resource "aws_lambda_function" "ws" {
  function_name    = "${var.service_name}-${var.stage}-ws"
  role             = aws_iam_role.lambda.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 10
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/ws.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/ws.zip")

  environment {
    variables = {
      ENV_PATH     = ".env.${var.stage}"
      RUST_LOG     = "info"
      DYNAMO_TABLE = aws_dynamodb_table.translations.name
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.basic_execution,
    aws_cloudwatch_log_group.ws,
  ]
}

resource "aws_lambda_function" "cron" {
  function_name    = "${var.service_name}-${var.stage}-cron"
  role             = aws_iam_role.lambda.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 900
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/cron.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/cron.zip")

  environment {
    variables = {
      ENV_PATH     = ".env.${var.stage}"
      RUST_LOG     = "info"
      DYNAMO_TABLE = aws_dynamodb_table.translations.name
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.basic_execution,
    aws_cloudwatch_log_group.cron,
  ]
}
