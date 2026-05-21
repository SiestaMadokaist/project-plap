resource "aws_cloudwatch_log_group" "api" {
  name              = "/aws/lambda/${var.service_name}-${var.stage}-api"
  retention_in_days = var.log_retention_days
}

resource "aws_cloudwatch_log_group" "ws" {
  name              = "/aws/lambda/${var.service_name}-${var.stage}-ws"
  retention_in_days = var.log_retention_days
}

resource "aws_cloudwatch_log_group" "cron" {
  name              = "/aws/lambda/${var.service_name}-${var.stage}-cron"
  retention_in_days = var.log_retention_days
}
