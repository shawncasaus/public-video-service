use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub upstreams: HashMap<String, String>,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    3000
}
fn default_request_timeout_ms() -> u64 {
    3000
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let _ = dotenvy::dotenv();

        let cfg = config::Config::builder()
            .set_default("host", default_host())?
            .set_default("port", default_port())?
            .set_default("request_timeout_ms", default_request_timeout_ms())?
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::File::with_name("crates/api-gateway/config").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        Ok(cfg.try_deserialize()?)
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

mod tests {
    use super::*;

    #[test]
    fn env_overrides_work() {
        let _ = dotenvy::from_filename_override(".env.does_not_exist");
        std::env::set_var("APP__HOST", "127.0.0.2");
        std::env::set_var("APP__PORT", "4000");
        let cfg = AppConfig::load().unwrap();
        assert_eq!(cfg.host, "127.0.0.2");
        assert_eq!(cfg.port, 4000);
    }
}


