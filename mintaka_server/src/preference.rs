use anyhow::{anyhow, bail};
use rusty_renju::utils::byte_size::ByteSize;
use std::env::Args;

#[derive(Default, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Clone)]
pub struct Preference {
    pub address: String,
    pub cores: usize,
    pub memory_limit: Option<ByteSize>,
    pub tls_config: Option<TlsConfig>,
}

impl Default for Preference {

    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_string(),
            cores: num_cpus::get_physical(),
            memory_limit: None,
            tls_config: None,
        }
    }

}

impl Preference {

    pub fn from_args(mut args: Args) -> anyhow::Result<Self> {
        let mut preference = Self::default();

        let mut tls_cert = None;
        let mut tls_key = None;

        let _ = args.next();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--address" | "-a" => {
                    if let Some(value) = args.next() {
                        preference.address = value;
                    } else {
                        bail!("missing value for --address argument");
                    }
                },
                "--cores" | "-c" => {
                    if let Some(value) = args.next() {
                        if value != "auto" {
                            preference.cores = value.parse::<usize>()
                                .map_err(|_| anyhow!("invalid cores value: {}", value))?;
                        }
                    } else {
                        bail!("missing value for --cores argument");
                    }
                },
                "--memory-limit" | "-m" => {
                    if let Some(value) = args.next() {
                        preference.memory_limit = Some(ByteSize::from_mib(
                            value.parse::<usize>()
                                .map_err(|_| anyhow!("invalid memory limit value: {}", value))?
                        ));
                    } else {
                        bail!("missing value for --memory-limit argument");
                    }
                }
                "--tls-cert" | "-tc" => {
                    if let Some(value) = args.next() {
                        tls_cert = Some(value);
                    } else {
                        bail!("missing value for --tls-cert argument");
                    }
                },
                "--tls-key" | "-tk" => {
                    if let Some(value) = args.next() {
                        tls_key = Some(value);
                    } else {
                        bail!("missing value for --tls-key argument");
                    }
                },
                _ => bail!("unknown argument: {}", arg),
            }
        }

        match (tls_cert, tls_key) {
            (Some(cert), Some(key)) => {
                preference.tls_config = Some(TlsConfig {
                    cert_path: cert,
                    key_path: key,
                });
            },
            (None, None) => {},
            _ => bail!("specific --tls-cert and --tls-key together to enable TLS")
        }

        Ok(preference)
    }

}
