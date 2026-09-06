output "api_endpoint" {
  value = aws_apigatewayv2_api.http.api_endpoint
}

output "api_url" {
  description = "Public API origin behind the custom domain — consumed by `make frontend-build`."
  value       = "https://${var.api_domain}"
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

output "frontend_url" {
  value = "https://${var.frontend_domain}"
}

output "frontend_bucket" {
  value = aws_s3_bucket.frontend.bucket
}

output "frontend_distribution_id" {
  value = aws_cloudfront_distribution.frontend.id
}
