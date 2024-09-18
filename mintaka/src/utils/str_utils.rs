pub fn join_str_horizontally(sources: &[&str]) -> String {
    let split = sources.iter()
        .map(|source|
            source
                .split("\n")
                .collect::<Box<[&str]>>()
        )
        .collect::<Box<[Box<[&str]>]>>();

    let max_len = split.iter()
        .flatten()
        .max_by_key(|row| row.len())
        .unwrap()
        .len();

    (0 .. split.first().unwrap().len())
        .map(|row_idx|
            split.iter()
                .map(|rows| {
                    let row = rows[row_idx].to_string();
                    let padding: String = " ".repeat(max_len - row.len());
                    format!("{row}{padding}")
                })
                .reduce(|head, tail|
                    format!("{head} {tail}")
                )
                .unwrap()
        )
        .reduce(|head, tail|
            format!("{head}\n{tail}")
        )
        .unwrap()
}
