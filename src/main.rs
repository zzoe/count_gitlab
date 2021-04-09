use std::panic::catch_unwind;
use std::thread;

use async_executor::Executor;
use futures_lite::{future, StreamExt};
use log::info;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use util::Select;

use crate::config::{Config, ProjectId};
use async_channel::bounded;

mod config;
pub mod excel;
pub mod gitlab;
pub mod sqlite;

pub static CONFIG: Lazy<Config> = Lazy::new(config::init);
pub static EXECUTOR: Lazy<Executor> = Lazy::new(Executor::new);

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    for i in 1..CONFIG.threads {
        thread::Builder::new()
            .name(format!("thread-{}", i))
            .spawn(|| loop {
                catch_unwind(|| future::block_on(EXECUTOR.run(future::pending::<()>()))).ok();
            })
            .expect("cannot spawn executor thread");
    }

    info!("统计开始");
    future::block_on(EXECUTOR.run(run()));
    info!("统计结束");
}

async fn run() {
    let mut deal_tasks = Select(Vec::new());
    let (s, r) = bounded(CONFIG.concurrent);

    for id in CONFIG.project.keys() {
        deal_tasks.0.push(EXECUTOR.spawn(gitlab::deal_project(
            s.clone(),
            r.clone(),
            ProjectId(id.parse().unwrap()),
        )));
    }

    let conn = Connection::open(&*CONFIG.sqlite).expect("连接数据库失败");
    let mut stmt = sqlite::init(&conn);
    while let Some(logs) = deal_tasks.next().await {
        sqlite::insert(&mut stmt, logs);
    }
    info!("保存sqlite结束: {}个项目", CONFIG.project.len());

    excel::create();
}
