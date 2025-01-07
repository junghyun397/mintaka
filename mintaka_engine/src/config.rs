pub struct Config {
    pub player_vcf_depth: usize,
    pub ai_vcf_depth: usize,

    pub max_search_depth: usize,
    pub max_nodes: usize,

    pub time_limit_milliseconds: usize,

    pub workers: usize,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            player_vcf_depth: 256,
            ai_vcf_depth: 256,
            max_search_depth: 100,
            max_nodes: 1_000_000,
            time_limit_milliseconds: usize::MAX,
            workers: 1,
        }
    }

}
