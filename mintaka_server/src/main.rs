mod preference;
mod session;
mod unbounded_response_sender;

use crate::preference::Preference;
use std::env;
use std::str::FromStr;
use time::OffsetDateTime;

fn log_prefix() -> String {
    OffsetDateTime::now_utc().to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pref = Preference::from_args(env::args().collect()).unwrap_or_default();

    Ok(())
}
