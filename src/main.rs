use std::sync::Arc;

use crate::config::Config;

mod config;
mod statistic;

thread_local! {
pub static CONFIG: Arc<Config> = Arc::new( config::init().unwrap());
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("SMOL_THREADS", "");
    pretty_env_logger::init_timed();

    println!("{:?}", smol::block_on(statistic::run()));
}
