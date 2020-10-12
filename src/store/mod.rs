use anyhow::Result;

use crate::config::StoreType;
use crate::statistic::CodeStatistics;
use crate::CONFIG;

mod file;
pub mod sqlite;

pub async fn save(id: usize, code_statistics: CodeStatistics) -> Result<()> {
    match CONFIG.store_type {
        StoreType::File => file::save(id, code_statistics).await?,
        StoreType::SQLite => {}
    }

    Ok(())
}
