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
    #[arg(short, long, default_value = "default")]
    pub address: String,
    #[arg(short, long, default_value_t = num_cpus::get_physical())]
    pub cores: usize,
    #[arg(short, long, help = "Total memory limit in MiB")]
    memory_limit_mib: Option<u64>,
    #[arg(long, requires = "tls_key", help = "TLS certificate file path")]
    tls_cert: Option<String>,
    #[arg(long, requires = "tls_cert", help = "TLS key file path")]
    tls_key: Option<String>,
    #[arg(long, help = "Reload TLS certificate on SIGHUP")]
    tls_renew: bool,
    #[arg(short, default_value = "sessions", help = "Session storage directory")]
    pub sessions_directory: String,
    #[arg(long, env = "MINTAKA_API_PASSWORD", default_value = None)]
    pub api_password: Option<String>,
    #[clap(skip)]
    pub memory_limit: ByteSize,
    #[clap(skip)]
    pub tls_config: Option<TlsConfig>,
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
                self.address = "127.0.0.1:8443".to_string();
            } else {
                self.address = "127.0.0.1:8080".to_string();
            }
        }

        if let Some(cert_path) = &self.tls_cert && let Some(key_path) = &self.tls_key {
            self.tls_config = Some(TlsConfig {
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
                observe_sighup: self.tls_renew,
            });
        }

        self.max_config = std::fs::read_to_string("max_config.toml")
            .ok()
            .and_then(|str| toml::from_str(&str).ok());
    }

}
