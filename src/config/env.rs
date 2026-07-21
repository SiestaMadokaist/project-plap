use std::env;

pub struct Env {
    pub aws_region: String,
    pub aws_default_region: String,
    pub translation_table: String,
    pub user_table: String,
    pub openai_model: String,
    pub syosetu_host: String,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub tl_bucket: String,
    pub tl_prefix: String,
    pub discord_webhook_url: String,
}

impl Env {
    pub fn from_env() -> Self {
        Self {
            aws_region: var_or("AWS_REGION", ""),
            aws_default_region: var_or("AWS_DEFAULT_REGION", ""),
            translation_table: var_or("TRANSLATION_TABLE", "production-translations"),
            user_table: var_or("USER_TABLE", "production-users"),
            openai_model: var_or("OPENAI_MODEL", "gpt-4o"),
            syosetu_host: var_or("SYOSETU_HOST", "https://ncode.syosetu.com"),
            proxy_host: env::var("PROXY_HOST").ok(),
            proxy_port: env::var("PROXY_PORT").ok(),
            proxy_username: env::var("PROXY_USERNAME").ok(),
            proxy_password: env::var("PROXY_PASSWORD").ok(),
            tl_bucket: env::var("TL_BUCKET").expect("TL_BUCKET must be set"),
            tl_prefix: var_or("TL_PREFIX", ""),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .expect("DISCORD_WEBHOOK_URL must be set"),
        }
    }

    pub fn region(&self) -> String {
        if !self.aws_region.is_empty() {
            self.aws_region.clone()
        } else if !self.aws_default_region.is_empty() {
            self.aws_default_region.clone()
        } else {
            "ap-southeast-1".to_string()
        }
    }
}

fn var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
