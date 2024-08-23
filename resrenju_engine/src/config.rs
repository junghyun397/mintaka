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
