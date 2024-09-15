mod args;

use crate::args::Preference;
use std::env;
use std::str::FromStr;
use time::OffsetDateTime;

fn log_prefix() -> String {
    OffsetDateTime::now_utc().to_string()
}

fn main() {
    let pref = Preference::from_args(env::args().collect()).unwrap_or_default();

    let server = tiny_http::Server::http(format!("0.0.0.0:{}", pref.port)).unwrap();

    println!("mintaka web-ui backend now listening on port {}.", pref.port);

    loop {
        let request = match server.recv() {
            Ok(request) => request,
            _ => break
        };

        if pref.verbose_mode {
            println!("{} income request: {:?}", log_prefix(), request);
        }
    }
}
