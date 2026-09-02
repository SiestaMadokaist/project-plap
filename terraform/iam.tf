locals {
  account_id = data.aws_caller_identity.current.account_id

  lambda_assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

# =========================================================================
# api
# =========================================================================

resource "aws_iam_role" "api" {
  name               = "${var.service_name}-${var.stage}-api-role"
  assume_role_policy = local.lambda_assume_role_policy
}

resource "aws_iam_role_policy_attachment" "api_basic_execution" {
  role       = aws_iam_role.api.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "api" {
  name = "${var.service_name}-${var.stage}-api-policy"
  role = aws_iam_role.api.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # CPModel::run -> AgentCommandRepository::insert (cp_model.rs) is PutItem-only today, but api is
        # meant to manage the queue directly (reorder priority, delete, push new), not just push - so this
        # grants the full set now even though the reorder/delete routes/usecases aren't wired yet. Query
        # needs the stage-priority GSI resource ARN in addition to the base table ARN.
        Sid    = "AgentCommandsReadWrite"
        Effect = "Allow"
        Action = ["dynamodb:PutItem", "dynamodb:Query", "dynamodb:UpdateItem", "dynamodb:DeleteItem"]
        Resource = [
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-agent_commands",
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-agent_commands/index/stage-priority",
        ]
      },
      {
        # DDBUserRepository (user.rs), wired into ApiRepos (bin/api/bootstrap/repo.rs): get/put/delete
        # by `username` on the base table; find/login look up by wallet address via Query on the
        # `address` GSI, so that index ARN is needed alongside the base table.
        Sid    = "UsersReadWrite"
        Effect = "Allow"
        Action = ["dynamodb:GetItem", "dynamodb:PutItem", "dynamodb:UpdateItem", "dynamodb:DeleteItem", "dynamodb:Query"]
        Resource = [
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-users",
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-users/index/address",
        ]
      },
      {
        # NOTE: no "presets" table exists yet (not in dynamodb.tf, no DDBTable variant, no code path).
        # Add the aws_dynamodb_table resource before this grant does anything.
        Sid      = "PresetsReadWrite"
        Effect   = "Allow"
        Action   = ["dynamodb:PutItem", "dynamodb:Query"]
        Resource = ["arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-presets"]
      },
      {
        # GetListModel::run -> storage.ls() on the model bucket (list.rs). List only, no object access needed.
        Sid      = "ModelBucketList"
        Effect   = "Allow"
        Action   = ["s3:ListBucket"]
        Resource = ["arn:aws:s3:::virginia-ramadoka"]
      },
      {
        # ap3.ramadoka.com = OUTPUT_BUCKET (ApiClients, output_storage), objects written with PublicRead ACL.
        # v.ramadoka.com is not referenced by any current .env.* file or code path - confirm this is still
        # actually needed before applying.
        Sid    = "OutputBucketReadWrite"
        Effect = "Allow"
        Action = ["s3:GetObject", "s3:PutObject", "s3:PutObjectAcl", "s3:ListBucket"]
        Resource = [
          "arn:aws:s3:::ap3.ramadoka.com",
          "arn:aws:s3:::ap3.ramadoka.com/*",
          "arn:aws:s3:::v.ramadoka.com",
          "arn:aws:s3:::v.ramadoka.com/*",
        ]
      },
      {
        # NOTE: ManageCompute usecase exists but isn't wired into routes() in bin/api/main.rs yet, and
        # ComputeEngine::{stop,launch,terminate,reboot} are all todo!(). Spot actions added per request
        # for "in case I can spawn spot instance" - RunInstances alone (with InstanceMarketOptions) is
        # enough for spot-at-launch; the RequestSpotInstances family is only needed for the older
        # standalone Spot Request workflow. Keep both until you pick one.
        Sid    = "ComputeControl"
        Effect = "Allow"
        Action = [
          "ec2:DescribeInstances",
          "ec2:StartInstances",
          "ec2:StopInstances",
          "ec2:TerminateInstances",
          "ec2:RebootInstances",
          "ec2:RunInstances",
          "ec2:CreateTags",
          "ec2:RequestSpotInstances",
          "ec2:CancelSpotInstanceRequests",
          "ec2:DescribeSpotInstanceRequests",
          "ec2:DescribeSpotPriceHistory",
        ]
        Resource = ["*"]
      },
      {
        # Pairs with ComputeControl above - only needed once api actually launches instances with the
        # diffusion-agent instance profile attached.
        Sid      = "PassRoleForSpotLaunch"
        Effect   = "Allow"
        Action   = ["iam:PassRole"]
        Resource = [aws_iam_role.diffusion_agent.arn]
      },
    ]
  })
}

# =========================================================================
# ws
# =========================================================================

resource "aws_iam_role" "ws" {
  name               = "${var.service_name}-${var.stage}-ws-role"
  assume_role_policy = local.lambda_assume_role_policy
}

resource "aws_iam_role_policy_attachment" "ws_basic_execution" {
  role       = aws_iam_role.ws.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "ws" {
  name = "${var.service_name}-${var.stage}-ws-policy"
  role = aws_iam_role.ws.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # NOTE: bin/ws/main.rs handler is currently a stub ($connect/$disconnect match arms are empty) -
        # no code calls PostToConnection/ManageConnections yet. Kept because it was already granted before
        # this split; drop it if the push-to-client feature isn't happening soon.
        Sid      = "ManageConnections"
        Effect   = "Allow"
        Action   = ["execute-api:ManageConnections"]
        Resource = ["arn:aws:execute-api:${var.region}:${local.account_id}:${aws_apigatewayv2_api.ws.id}/*"]
      },
    ]
  })
}

# =========================================================================
# cron
# =========================================================================

resource "aws_iam_role" "cron" {
  name               = "${var.service_name}-${var.stage}-cron-role"
  assume_role_policy = local.lambda_assume_role_policy
}

resource "aws_iam_role_policy_attachment" "cron_basic_execution" {
  role       = aws_iam_role.cron.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "cron" {
  name = "${var.service_name}-${var.stage}-cron-policy"
  role = aws_iam_role.cron.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # translations/run.rs, translations/init.rs -> TranslationRepository (translation.rs: put_item, query
        # against the base table only, no index_name() call, so no GSI/LSI resource ARN needed).
        Sid      = "TranslationsWrite"
        Effect   = "Allow"
        Action   = ["dynamodb:PutItem", "dynamodb:Query"]
        Resource = ["arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-translations"]
      },
      {
        # translations/run.rs -> output_storage.write() (run.rs:114). TL_BUCKET in .env.production.
        Sid      = "TranslationOutputWrite"
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:PutObjectAcl"]
        Resource = ["arn:aws:s3:::stardust-frontiers/project-translation/*"]
      },
      {
        # NOTE: HotReloadRepository::diffusion_config() is todo!() - unimplemented, nothing calls this yet.
        Sid      = "HotReloadRead"
        Effect   = "Allow"
        Action   = ["dynamodb:GetItem"]
        Resource = ["arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-hot_reloads"]
      },
    ]
  })
}

# =========================================================================
# diffusion-agent (EC2 instance profile - not a Lambda role)
# =========================================================================

resource "aws_iam_role" "diffusion_agent" {
  name = "${var.service_name}-${var.stage}-diffusion-agent-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ec2.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

# NOTE: no aws_instance/launch_template resource exists in this terraform config yet - the box itself is
# still managed by hand (see `make deploy-bin`). Attach this profile to it manually until that's automated.
resource "aws_iam_instance_profile" "diffusion_agent" {
  name = "${var.service_name}-${var.stage}-diffusion-agent-profile"
  role = aws_iam_role.diffusion_agent.name
}

resource "aws_iam_role_policy" "diffusion_agent" {
  name = "${var.service_name}-${var.stage}-diffusion-agent-policy"
  role = aws_iam_role.diffusion_agent.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # CommandQ::on_interval (query, via the stage-priority GSI) + CommandHandler::record_progress
        # (set_progress: get() [base-table query] + put_item on the failed/done paths, update_item on the
        # in-progress path). agent_command.rs.
        Sid    = "AgentCommandsReadWrite"
        Effect = "Allow"
        Action = ["dynamodb:Query", "dynamodb:PutItem", "dynamodb:UpdateItem"]
        Resource = [
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-agent_commands",
          "arn:aws:dynamodb:${var.region}:${local.account_id}:table/${var.stage}-agent_commands/index/stage-priority",
        ]
      },
      {
        # EC2DiffusionClients::new - model_storage (bootstrap/client.rs). MODEL_BUCKET in .env.diffusion.
        Sid      = "ModelBucketList"
        Effect   = "Allow"
        Action   = ["s3:ListBucket"]
        Resource = ["arn:aws:s3:::virginia-ramadoka"]
      },
      {
        Sid      = "ModelBucketObjectReadWrite"
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject"]
        Resource = ["arn:aws:s3:::virginia-ramadoka/*"]
      },
      {
        # EC2DiffusionClients::new - output_storage, PublicRead ACL. OUTPUT_BUCKET in .env.diffusion.
        Sid      = "OutputBucketWrite"
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:PutObjectAcl"]
        Resource = ["arn:aws:s3:::ap3.ramadoka.com/*"]
      },
      {
        # NOTE: ComputeEngine::{stop,terminate,reboot} are todo!() - ManageCompute isn't called anywhere yet.
        # No ec2:StartInstances here on purpose - a stopped instance can't call StartInstances on itself;
        # that's the api role's job (see ComputeControl above), triggered externally.
        # NOTE on "handle if own self is spot instances": that's an IMDS concern, not an IAM one - spot
        # interruption notices come from http://169.254.169.254/latest/meta-data/spot/instance-action,
        # same unauthenticated metadata endpoint EC2Agent already polls for ip()/instance_id()/region() in
        # infras/compute_agent/ec2.rs. No extra IAM action covers "detect my own spot termination"; added
        # none here. If you instead want the agent to check its own state via the EC2 API rather than IMDS,
        # DescribeInstances (below) already covers that.
        Sid      = "SelfComputeControl"
        Effect   = "Allow"
        Action   = ["ec2:DescribeInstances", "ec2:StopInstances", "ec2:TerminateInstances", "ec2:RebootInstances"]
        Resource = ["*"]
      },
    ]
  })
}
