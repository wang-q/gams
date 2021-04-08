use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u32,
    pub redis_password: Option<String>,
    #[serde(default = "default_redis_tls")]
    pub redis_tls: bool,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u32 {
    6379
}

fn default_redis_tls() -> bool {
    false
}
