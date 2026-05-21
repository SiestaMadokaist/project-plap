locals {
  cron_schedules = {
    ocr_service_morning = {
      schedule    = "cron(0 2 * * ? *)"
      description = "OCR service 02:00 UTC daily"
      input       = jsonencode({ pathParameters = { proxy = "cron/ocr-service" }, requestContext = { http = { method = "POST" } } })
    }
    ocr_service_afternoon = {
      schedule    = "cron(0 14 * * ? *)"
      description = "OCR service 14:00 UTC daily"
      input       = jsonencode({ pathParameters = { proxy = "cron/ocr-service" }, requestContext = { http = { method = "POST" } } })
    }
    mldn = {
      schedule    = "cron(0 1 * * ? *)"
      description = "MLDN 01:00 UTC daily"
      input       = jsonencode({ pathParameters = { proxy = "cron/mldn" }, requestContext = { http = { method = "POST" } } })
    }
    autosync = {
      schedule    = "cron(0 13 * * ? *)"
      description = "Autosync 13:00 UTC daily"
      input       = jsonencode({ pathParameters = { proxy = "cron/autosync" }, requestContext = { http = { method = "POST" } } })
    }
  }
}

resource "aws_cloudwatch_event_rule" "cron" {
  for_each            = local.cron_schedules
  name                = "${var.service_name}-${var.stage}-${each.key}"
  description         = each.value.description
  schedule_expression = each.value.schedule
  state               = "ENABLED"
}

resource "aws_cloudwatch_event_target" "cron" {
  for_each  = local.cron_schedules
  rule      = aws_cloudwatch_event_rule.cron[each.key].name
  target_id = "cron-lambda"
  arn       = aws_lambda_function.cron.arn
  input     = each.value.input
}

resource "aws_lambda_permission" "cron" {
  for_each      = local.cron_schedules
  statement_id  = "AllowEventBridgeInvoke-${each.key}"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.cron.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.cron[each.key].arn
}
