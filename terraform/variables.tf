variable "region" {
  default = "ap-southeast-1"
}

variable "stage" {
  default = "production"
}

variable "service_name" {
  default = "rust-api"
}

variable "deploy_bucket" {
  default = "serverless.ramadoka.com"
}

variable "memory_size" {
  default = 512
}

variable "log_retention_days" {
  default = 7
}

variable "root_domain" {
  description = "Existing Route53 public hosted zone"
  default     = "ramadoka.com"
}

variable "frontend_domain" {
  description = "Hostname that serves the Leptos frontend"
  default     = "plap.ramadoka.com"
}

variable "api_domain" {
  description = "Custom domain for the HTTP API"
  default     = "plap-api.ramadoka.com"
}

variable "frontend_bucket_name" {
  description = "S3 bucket for frontend assets (no dots — CloudFront OAC dislikes dotted bucket names)"
  default     = "plap-frontend-production"
}

variable "frontend_price_class" {
  description = "CloudFront price class. PriceClass_All keeps the Jakarta + Singapore edge locations active for lowest latency from Indonesia."
  default     = "PriceClass_All"
}
