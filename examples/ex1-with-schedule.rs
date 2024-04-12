use std::fmt::Error;
use tokio::runtime::Runtime;

use rust_cronjob_ollama::consts::CRON_EXPRESSION_5_MIN;
use rust_cronjob_ollama::cron_util;
use rust_cronjob_ollama::ollama_helper::get_joke;

pub type Result<T> = core::result::Result<T, Error>;

fn main() {
    cron_util::create_cronjob_with_schedule(CRON_EXPRESSION_5_MIN, produce_joke);
}

fn produce_joke() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        match get_joke().await {
            Ok(joke) => println!("{}", joke),
            Err(e) => println!("Error: {}", e),
        }
    });
}