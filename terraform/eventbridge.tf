locals {
  cron_schedules = {
    translate_deathflag = {
      schedule    = "cron(30 * * * ? *)"
      description = "Translate deathflag every hour at :30"
      input       = jsonencode({ pathParameters = { proxy = "cron/translate", novel_id = "n4449cj" } })
    }
    translate_re0 = {
      schedule    = "cron(0 * * * ? *)"
      description = "Translate rezero every hour at :00"
      input       = jsonencode({ pathParameters = { proxy = "cron/translate", novel_id = "n2267be" } })
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
