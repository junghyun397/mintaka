pub fn join_str_horizontally(sources: &[&str]) -> String {
    let split: Vec<Vec<(usize, &str)>> = sources.into_iter()
        .map(|source|
            source
                .split("\n")
                .enumerate()
                .collect::<Vec<(usize, &str)>>()
        )
        .collect::<Vec<Vec<(usize, &str)>>>();

    let max_len = split.iter()
        .flatten()
        .max_by_key(|(_, row)| row.len())
        .unwrap()
        .1.len();

    (0 .. split.first().unwrap().len()).into_iter()
        .map(|idx|
            split.iter()
                .map(|row| {
                    let raw = row[idx].1.to_string();
                    let padding: String = std::iter::repeat(' ')
                        .take(max_len - raw.len())
                        .collect();

                    format!("{}{}", raw, padding)
                })
                .reduce(|head, tail|
                    format!("{} {}", head, tail)
                )
                .unwrap()
        )
        .reduce(|head, tail|
            format!("{}\n{}", head, tail)
        )
        .unwrap()
}
