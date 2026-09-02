resource "aws_apigatewayv2_api" "http" {
  name          = "${var.service_name}-${var.stage}-http"
  protocol_type = "HTTP"

  # API Gateway answers the OPTIONS preflight itself (never invokes the Lambda) and
  # injects these headers on every response. Scoped to the frontend origin, which
  # API Gateway echoes back on a match. `authorization` must be listed explicitly —
  # the Fetch spec doesn't let `*` cover it.
  cors_configuration {
    allow_origins = ["https://${var.frontend_domain}"]
    allow_methods = ["*"]
    allow_headers = ["authorization", "content-type"]
    max_age       = 86400
  }
}

resource "aws_apigatewayv2_integration" "api" {
  api_id                 = aws_apigatewayv2_api.http.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.api.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "api_default" {
  api_id    = aws_apigatewayv2_api.http.id
  route_key = "$default"
  target    = "integrations/${aws_apigatewayv2_integration.api.id}"
}

resource "aws_apigatewayv2_stage" "http" {
  api_id      = aws_apigatewayv2_api.http.id
  name        = "$default"
  auto_deploy = true


}


resource "aws_lambda_permission" "api_http" {
  statement_id  = "AllowHTTPAPIInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.api.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http.execution_arn}/*/*"
}
