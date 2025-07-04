pub struct Preference {
    pub port: usize,
    pub verbose_output: bool,
    pub cores: usize,
}

impl Default for Preference {

    fn default() -> Self {
        Self {
            port: 8000,
            verbose_output: false,
            cores: 1,
        }
    }

}

impl Preference {

    pub fn from_args(source: Vec<String>) -> Result<Self, &'static str> {
        todo!()
    }

}
