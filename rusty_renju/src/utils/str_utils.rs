use crate::const_for;

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
                    let mut row = rows[row_idx].to_string();
                    row.extend(std::iter::repeat_n(' ', max_len - row.len()));
                    row
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
        .collect::<Vec<_>>()
        .join("\n")
}

pub const fn u8_from_str(source: &str, skip: usize) -> u8 {
    let bytes = source.as_bytes();
    let mut result = 0u8;

    const_for!(idx in skip, bytes.len(); {
        let byte = bytes[idx];
        if byte >= b'0' && byte <= b'9' {
            result = result * 10 + (byte - b'0');
        } else {
            panic!("Invalid character in source");
        }
    });

    result
}
