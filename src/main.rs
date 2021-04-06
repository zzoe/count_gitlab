use std::panic::catch_unwind;
use std::thread;

use anyhow::Result;
use async_executor::Executor;
use futures_lite::{future, StreamExt};
use log::{error, info};
use once_cell::sync::Lazy;
use util::Select;

use crate::config::{Config, ProjectId};

mod config;
pub mod excel;
pub mod gitlab;
pub mod sqlite;

pub static CONFIG: Lazy<Config> = Lazy::new(|| config::init().expect("读取配置文件失败"));
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
    if let Err(e) = future::block_on(EXECUTOR.run(run())) {
        error!("统计失败: {}", e)
    }
}

async fn run() -> Result<()> {
    let mut tasks = Select(Vec::new());

    for id in CONFIG.project.keys() {
        tasks
            .0
            .push(EXECUTOR.spawn(gitlab::deal_project(ProjectId(id.parse()?))));
    }

    let conn = sqlite::connect()?;
    let mut stmt = sqlite::prepare_insert(&conn)?;

    let mut fail = 0_u32;
    while let Some(res) = tasks.next().await {
        if let Err(e) = res.and_then(|logs| sqlite::insert(&mut stmt, logs)) {
            error!("保存项目提交记录失败: {}", e);
            fail += 1;
        }
    }

    info!("统计结束: {}个项目，失败{}个 ", CONFIG.project.len(), fail);

    if fail == 0 {
        excel::create(&conn)?;
        info!("写入excel成功");
    }

    Ok(())
}
