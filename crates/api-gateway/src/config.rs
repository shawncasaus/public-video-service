use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_port() -> u16 {
    3000
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let _ = dotenvy::dotenv();
        let cfg = ::config::Config::builder()
            .set_default("host", default_host())?
            .set_default("port", default_port())?
            .add_source(::config::File::with_name("config").required(false))
            .add_source(::config::Environment::with_prefix("APP").separator("_"))
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
