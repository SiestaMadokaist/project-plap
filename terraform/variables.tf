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

variable "discord_webhook_url" {
  description = "Discord webhook URL for translation notifications"
  sensitive   = true
}
