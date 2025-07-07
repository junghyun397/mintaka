use anyhow::bail;
use std::env::Args;

pub struct Preference {
    pub address: String,
    pub cores: usize,
}

impl Default for Preference {

    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_string(),
            cores: num_cpus::get_physical(),
        }
    }

}

impl Preference {

    pub fn from_args(mut args: Args) -> anyhow::Result<Self> {
        let mut preference = Self::default();

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
                                .map_err(|_| anyhow::anyhow!("Invalid cores value: {}", value))?;
                        }
                    } else {
                        bail!("missing value for --cores argument");
                    }
                },
                _ => bail!("Unknown argument: {}", arg),
            }
        }

        Ok(preference)
    }

}
