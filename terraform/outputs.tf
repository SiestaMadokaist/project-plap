output "api_endpoint" {
  value = aws_apigatewayv2_api.http.api_endpoint
}

output "ws_endpoint" {
  value = "${aws_apigatewayv2_api.ws.api_endpoint}/${var.stage}"
}

output "lambda_api_arn" {
  value = aws_lambda_function.api.arn
}

output "lambda_ws_arn" {
  value = aws_lambda_function.ws.arn
}

output "lambda_cron_arn" {
  value = aws_lambda_function.cron.arn
}
