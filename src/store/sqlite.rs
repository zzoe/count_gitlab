use anyhow::Result;
use rbatis::crud::{CRUDEnable, CRUD};
use rbatis::rbatis::Rbatis;
use serde::{Deserialize, Serialize};

use crate::statistic::Record;

pub async fn save(id: usize, record: Vec<Commits>) -> Result<()> {
    let rb = Rbatis::new();
    rb.link("sqlite://gitlab.db").await?;

    Ok(())
}

#[derive(CRUDEnable, Serialize, Deserialize, Clone, Debug)]
pub struct Commits {
    id: String,
    short_id: String,
    author_name: String,
    author_email: String,
    authored_date: String,
    committer_name: String,
    committer_email: String,
    committed_date: String,
    created_at: String,
    title: String,
    message: String,
    parent_ids: Vec<String>,
    additions: usize,
    deletions: usize,
}

impl From<Record> for Commits {
    fn from(record: Record) -> Self {
        Commits {
            id: record.id,
            short_id: record.short_id,
            author_name: record.author_name,
            author_email: record.author_email,
            authored_date: record.authored_date,
            committer_name: record.committer_name,
            committer_email: record.committer_email,
            committed_date: record.committed_date,
            created_at: record.created_at,
            title: record.title,
            message: record.message,
            parent_ids: record.parent_ids,
            additions: record.stats.additions,
            deletions: record.stats.deletions,
        }
    }
}
