use std::env::Args;

pub struct Preference {
    pub address: String,
    pub verbose_output: bool,
    pub cores: usize,
}

impl Default for Preference {

    fn default() -> Self {
        Self {
            address: "localhost:8080".to_string(),
            verbose_output: false,
            cores: 1,
        }
    }

}

impl Preference {

    pub fn from_args(args: Args) -> Result<Self, &'static str> {
        let preference = Self::default();

        Ok(preference)
    }

}
