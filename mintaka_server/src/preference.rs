use argh::FromArgs;
use mintaka::config::Config;
use rusty_renju::utils::byte_size::ByteSize;
use std::str::FromStr;

#[derive(Default, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub observe_sighup: bool,
}

#[derive(Clone)]
pub struct Preference {
    pub webui: bool,
    pub open_webui: bool,
    pub address: String,
    pub cores: usize,
    pub sessions_directory: String,
    pub api_password: Option<String>,
    pub memory_limit: ByteSize,
    pub tls_config: Option<TlsConfig>,
    pub default_config: Config,
    pub max_config: Option<Config>
}

#[derive(FromArgs)]
#[argh(description = "mintaka web API provider", help_triggers("-h", "--help"))]
struct Args {
    #[argh(switch, description = "serve the web UI [env: WEBUI]")]
    webui: Option<bool>,
    #[argh(switch, description = "open the web UI in a browser")]
    open_webui: bool,
    #[argh(option, short = 'a', description = "listen address [env: ADDRESS]")]
    address: Option<String>,
    #[argh(option, short = 'c', description = "number of CPU cores [env: CORES]")]
    cores: Option<usize>,
    #[argh(option, short = 'm', description = "total memory limit in MiB [env: MEMORY_LIMIT_MIB]")]
    memory_limit_mib: Option<u64>,
    #[argh(option, description = "TLS certificate file path [env: TLS_CERT]")]
    tls_cert: Option<String>,
    #[argh(option, description = "TLS key file path [env: TLS_KEY]")]
    tls_key: Option<String>,
    #[argh(switch, description = "reload TLS certificate on SIGHUP [env: TLS_RENEW]")]
    tls_renew: Option<bool>,
    #[argh(option, short = 's', default = "String::from(\"sessions\")", description = "session storage directory")]
    sessions_directory: String,
    #[argh(option, description = "password required to create sessions [env: API_PASSWORD]")]
    api_password: Option<String>,
}

impl Preference {
    pub fn parse() -> Self {
        argh::from_env::<Args>().try_into().unwrap_or_else(|error| {
            eprintln!("{error}\nRun --help for more information.");
            std::process::exit(1);
        })
    }

    fn parse_config(path: &str) -> Option<Config> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|str| toml::from_str(&str).ok())
    }
}

impl TryFrom<Args> for Preference {
    type Error = String;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let tls_cert = option_or_env(args.tls_cert, "TLS_CERT");
        let tls_key = option_or_env(args.tls_key, "TLS_KEY");
        let tls_renew = option_or_env(args.tls_renew, "TLS_RENEW").unwrap_or(false);

        let tls_config = match (tls_cert, tls_key) {
            (Some(cert_path), Some(key_path)) => Some(
                TlsConfig { cert_path, key_path, observe_sighup: tls_renew }
            ),
            (None, None) => None,
            _ => return Err("specific --tls-cert and --tls-key together".to_string()),
        };

        let address = option_or_env(args.address, "ADDRESS")
            .filter(|address| address != "default")
            .unwrap_or_else(|| if tls_config.is_some() {
                "0.0.0.0:8445".to_string()
            } else {
                "0.0.0.0:8085".to_string()
            });

        Ok(Self {
            webui: option_or_env(args.webui, "WEBUI").unwrap_or(false),
            open_webui: args.open_webui,
            address,
            cores: option_or_env(args.cores, "CORES").unwrap_or_else(num_cpus::get_physical),
            sessions_directory: args.sessions_directory,
            api_password: option_or_env(args.api_password, "API_PASSWORD"),
            memory_limit: option_or_env(args.memory_limit_mib, "MEMORY_LIMIT_MIB")
                .map(ByteSize::from_mib)
                .unwrap_or(ByteSize::from_mib(4096)),
            tls_config,
            default_config: Self::parse_config("default_config.toml").unwrap_or_default(),
            max_config: Self::parse_config("max_config.toml"),
        })
    }
}

fn option_or_env<T: FromStr>(option: Option<T>, name: &str) -> Option<T> {
    option.or_else(|| {
        std::env::var(name).ok()?
            .parse().ok()
    })
}
