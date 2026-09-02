locals {
  dist_dir = "${path.module}/../dist"
}

resource "aws_lambda_function" "api" {
  function_name    = "${var.service_name}-${var.stage}-api"
  role             = aws_iam_role.api.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 10
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/api.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/api.zip")

  environment {
    variables = {
      # api.zip bundles .env.api (not .env.${stage}) — it carries the auth/storage
      # config the other functions don't need. See the `package` target in the Makefile.
      ENV_PATH = ".env.api"
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.api_basic_execution,
    aws_cloudwatch_log_group.api,
  ]
}

resource "aws_lambda_function" "ws" {
  function_name    = "${var.service_name}-${var.stage}-ws"
  role             = aws_iam_role.ws.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 10
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/ws.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/ws.zip")

  environment {
    variables = {
      ENV_PATH = ".env.${var.stage}"
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.ws_basic_execution,
    aws_cloudwatch_log_group.ws,
  ]
}

resource "aws_lambda_function" "cron" {
  function_name    = "${var.service_name}-${var.stage}-cron"
  role             = aws_iam_role.cron.arn
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  handler          = "bootstrap"
  timeout          = 900
  memory_size      = var.memory_size
  filename         = "${local.dist_dir}/cron.zip"
  source_code_hash = filebase64sha256("${local.dist_dir}/cron.zip")

  environment {
    variables = {
      ENV_PATH = ".env.${var.stage}"
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.cron_basic_execution,
    aws_cloudwatch_log_group.cron,
  ]
}
