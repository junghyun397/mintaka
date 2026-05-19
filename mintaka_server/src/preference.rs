use std::time::Duration;
use clap::Parser;
use mintaka::config::Config;
use rusty_renju::utils::byte_size::ByteSize;

#[derive(Default, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub observe_sighup: bool,
}

fn version_str() -> &'static str {
    Box::leak(
        format!("rusty-renju={}, mintaka={}, mintaka_server={}",
            rusty_renju::VERSION,
            mintaka::VERSION,
            env!("CARGO_PKG_VERSION")
        ).into_boxed_str()
    )
}

#[derive(Clone, Parser)]
#[
    command(version = version_str(), disable_version_flag = true,
    author = "JeongHyeon Choi",
    about = "mintaka_server: mintaka web api provider.",
    long_about = None)
]
pub struct Preference {
    #[arg(long, env = "WEBUI", default_value = "false")]
    pub webui: bool,
    #[arg(long, default_value = "false")]
    pub open_webui: bool,
    #[arg(short, env = "ADDRESS", long, default_value = "default")]
    pub address: String,
    #[arg(short, long, default_value_t = num_cpus::get_physical())]
    pub cores: usize,
    #[arg(short, env = "MEMORY_LIMIT_MIB", long, help = "Total memory limit in MiB")]
    memory_limit_mib: Option<u64>,
    #[arg(long, env = "TLS_CERT", requires = "tls_key", help = "TLS certificate file path")]
    tls_cert: Option<String>,
    #[arg(long, env = "TLS_KEY", requires = "tls_cert", help = "TLS key file path")]
    tls_key: Option<String>,
    #[arg(long, env = "TLS_RENEW", help = "Reload TLS certificate on SIGHUP")]
    tls_renew: bool,
    #[arg(short, default_value = "sessions", help = "Session storage directory")]
    pub sessions_directory: String,
    #[arg(long, env = "API_PASSWORD", default_value = None)]
    pub api_password: Option<String>,
    #[arg(long, env = "TOKEN_SECRET", default_value = "verycomplexedsecret")]
    pub session_token_secret: String,
    #[arg(long, env = "HIBERNATE_TIMEOUT", default_value = "None")]
    pub hibernate_timeout_secs: Option<u64>,
    #[clap(skip)]
    pub hibernate_timeout: Option<Duration>,
    #[clap(skip)]
    pub memory_limit: ByteSize,
    #[clap(skip)]
    pub tls_config: Option<TlsConfig>,
    #[clap(skip)]
    pub default_config: Config,
    #[clap(skip)]
    pub max_config: Option<Config>
}

impl Preference {

    pub fn parse() -> Self {
        let mut pref = Self::parse_from(std::env::args());

        pref.init();

        pref
    }

    fn init(&mut self) {
        self.memory_limit = self.memory_limit_mib
            .map(ByteSize::from_mib)
            .unwrap_or(ByteSize::from_mib(4096));

        if &self.address == "default" {
            if self.tls_cert.is_some() {
                self.address = "127.0.0.1:8445".to_string();
            } else {
                self.address = "127.0.0.1:8085".to_string();
            }
        }

        if let Some(cert_path) = &self.tls_cert && let Some(key_path) = &self.tls_key {
            self.tls_config = Some(TlsConfig {
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
                observe_sighup: self.tls_renew,
            });
        }

        self.hibernate_timeout = self.hibernate_timeout_secs.map(Duration::from_secs);

        self.max_config = Self::parse_config("max_config.toml");
        self.default_config = Self::parse_config("default_config.toml").unwrap_or(Config::default());
    }

    fn parse_config(path: &str) -> Option<Config> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|str| toml::from_str(&str).ok())
    }

}
