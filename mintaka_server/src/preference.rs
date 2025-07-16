use clap::Parser;
use clap_derive::Parser;
use rusty_renju::utils::byte_size::ByteSize;

#[derive(Default, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub observe_sighup: bool,
}

fn version_str() -> &'static str {
    let version_str = format!("rusty-renju={}, mintaka={}, mintaka_server={}",
        rusty_renju::VERSION,
        mintaka::VERSION,
        env!("CARGO_PKG_VERSION")
    );

    Box::leak(version_str.into_boxed_str())
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
    memory_limit_mib: Option<usize>,
    #[arg(long, requires = "tls_key", help = "TLS certificate file path")]
    tls_cert: Option<String>,
    #[arg(long, requires = "tls_cert", help = "TLS key file path")]
    tls_key: Option<String>,
    #[arg(long, help = "Reload TLS certificate on SIGHUP")]
    tls_renew: bool,
    #[clap(skip)]
    pub memory_limit: Option<ByteSize>,
    #[clap(skip)]
    pub tls_config: Option<TlsConfig>,
}

impl Default for Preference {

    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_string(),
            cores: num_cpus::get_physical(),
            memory_limit_mib: None,
            tls_cert: None,
            tls_key: None,
            tls_renew: false,
            memory_limit: None,
            tls_config: None,
        }
    }

}

impl Preference {

    /// Parse command line arguments and create a Preference
    pub fn parse() -> anyhow::Result<Self> {
        let mut pref = Self::parse_from(std::env::args());
        pref.post_parse()?;
        Ok(pref)
    }

    /// Process parsed arguments and set up derived fields
    fn post_parse(&mut self) -> anyhow::Result<()> {
        if let Some(limit_mib) = self.memory_limit_mib {
            self.memory_limit = Some(ByteSize::from_mib(limit_mib));
        }

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

        Ok(())
    }

}