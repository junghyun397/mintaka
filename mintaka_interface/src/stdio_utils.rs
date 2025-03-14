pub fn read_line() -> Vec<String> {
    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        let args = buf.trim()
            .split(' ')
            .map(&str::to_string)
            .collect::<Vec<String>>();

        if args.len() == 0 {
            continue;
        }

        return args
    }
}
