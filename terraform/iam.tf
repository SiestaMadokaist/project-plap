locals {
  account_id = data.aws_caller_identity.current.account_id
}

resource "aws_iam_role" "lambda" {
  name = "${var.service_name}-${var.stage}-lambda-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "basic_execution" {
  role       = aws_iam_role.lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "lambda_policy" {
  name = "${var.service_name}-${var.stage}-policy"
  role = aws_iam_role.lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["ce:GetCostAndUsage"]
        Resource = ["arn:aws:ce:us-east-1:${local.account_id}:/GetCostAndUsage"]
      },
      {
        Effect = "Allow"
        Action = ["dynamodb:*"]
        Resource = [
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/production-*",
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/production-*/index/*",
        ]
      },
      {
        Effect   = "Allow"
        Action   = ["ec2:*"]
        Resource = ["*"]
      },
      {
        Effect   = "Allow"
        Action   = ["iam:PassRole"]
        Resource = ["*"]
      },
      {
        Effect = "Allow"
        Action = ["s3:*"]
        Resource = [
          "arn:aws:s3:::ap3.ramadoka.com",
          "arn:aws:s3:::ap3.ramadoka.com/images/*",
          "arn:aws:s3:::v.ramadoka.com",
          "arn:aws:s3:::v.ramadoka.com/prompts/*",
          "arn:aws:s3:::sydney.ramadoka.com",
          "arn:aws:s3:::sydney.ramadoka.com/*",
          "arn:aws:s3:::public.ramadoka.com",
          "arn:aws:s3:::public.ramadoka.com/*",
          "arn:aws:s3:::private.ramadoka.com",
          "arn:aws:s3:::private.ramadoka.com/*",
        ]
      },
      {
        Effect   = "Allow"
        Action   = ["execute-api:ManageConnections"]
        Resource = ["arn:aws:execute-api:${var.region}:${local.account_id}:${aws_apigatewayv2_api.ws.id}/*"]
      },
    ]
  })
}
