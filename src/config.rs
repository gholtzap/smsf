use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub enabled: bool,
    pub issuer: String,
    pub audience: Vec<String>,
    pub required_scope: Option<String>,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub client_ca_path: Option<String>,
    pub require_client_cert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sbi_bind_addr: String,
    pub sbi_bind_port: u16,
    pub mongodb_uri: String,
    pub nrf_uri: String,
    pub nf_instance_id: String,
    pub smsf_host: String,
    pub oauth2: OAuth2Config,
    pub tls: TlsConfig,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer: String::new(),
            audience: vec![],
            required_scope: None,
            secret_key: String::new(),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            client_cert_path: None,
            client_key_path: None,
            client_ca_path: None,
            require_client_cert: false,
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.apply_env_overrides();
        Ok(config)
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let sbi_bind_addr = env::var("SBI_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let sbi_bind_port = env::var("SBI_BIND_PORT")
            .unwrap_or_else(|_| "8085".to_string())
            .parse()?;

        let mongodb_uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let nrf_uri = env::var("NRF_URI")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());

        let nf_instance_id = env::var("NF_INSTANCE_ID")
            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let smsf_host = env::var("SMSF_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let oauth2_enabled = env::var("OAUTH2_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let oauth2_issuer = env::var("OAUTH2_ISSUER")
            .unwrap_or_else(|_| "".to_string());

        let oauth2_audience = env::var("OAUTH2_AUDIENCE")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let oauth2_required_scope = env::var("OAUTH2_REQUIRED_SCOPE").ok();

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "".to_string());

        let oauth2 = OAuth2Config {
            enabled: oauth2_enabled,
            issuer: oauth2_issuer,
            audience: oauth2_audience,
            required_scope: oauth2_required_scope,
            secret_key: jwt_secret,
        };

        let tls_enabled = env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let tls_cert_path = env::var("TLS_CERT_PATH").ok();
        let tls_key_path = env::var("TLS_KEY_PATH").ok();
        let tls_client_cert_path = env::var("TLS_CLIENT_CERT_PATH").ok();
        let tls_client_key_path = env::var("TLS_CLIENT_KEY_PATH").ok();
        let tls_client_ca_path = env::var("TLS_CLIENT_CA_PATH").ok();
        let tls_require_client_cert = env::var("TLS_REQUIRE_CLIENT_CERT")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let tls = TlsConfig {
            enabled: tls_enabled,
            cert_path: tls_cert_path,
            key_path: tls_key_path,
            client_cert_path: tls_client_cert_path,
            client_key_path: tls_client_key_path,
            client_ca_path: tls_client_ca_path,
            require_client_cert: tls_require_client_cert,
        };

        Ok(Self {
            sbi_bind_addr,
            sbi_bind_port,
            mongodb_uri,
            nrf_uri,
            nf_instance_id,
            smsf_host,
            oauth2,
            tls,
        })
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("SBI_BIND_ADDR") {
            self.sbi_bind_addr = val;
        }
        if let Ok(val) = env::var("SBI_BIND_PORT") {
            if let Ok(port) = val.parse() {
                self.sbi_bind_port = port;
            }
        }
        if let Ok(val) = env::var("MONGODB_URI") {
            self.mongodb_uri = val;
        }
        if let Ok(val) = env::var("NRF_URI") {
            self.nrf_uri = val;
        }
        if let Ok(val) = env::var("NF_INSTANCE_ID") {
            self.nf_instance_id = val;
        }
        if let Ok(val) = env::var("SMSF_HOST") {
            self.smsf_host = val;
        }
    }
}
