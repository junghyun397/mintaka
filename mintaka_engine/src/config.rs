pub struct Config {
    player_vcf_depth: usize,
    player_vct_depth: usize,

    ai_vcf_depth: usize,
    ai_vct_depth: usize,

    max_search_depth: usize,
    max_nodes: usize,

    time_limit_milliseconds: usize,

    workers: usize,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            player_vcf_depth: 256,
            player_vct_depth: 256,
            ai_vcf_depth: 256,
            ai_vct_depth: 256,
            max_search_depth: 100,
            max_nodes: 1_000_000,
            time_limit_milliseconds: usize::MAX,
            workers: 1,
        }
    }

}
