use anyhow::Result;
use log::warn;
use rbatis::crud::{CRUDEnable, CRUD};
use serde::{Deserialize, Serialize};

use crate::statistic::Record;
use crate::RB;

pub async fn save(records: Vec<CommitLog>) -> Result<()> {
    let mut count = 0;
    let rb = RB.lock().await;

    rb.begin("commit_log").await.expect("begin error");

    for record in &records {
        count += rb.save("", record).await?;
    }

    rb.commit("commit_log").await.expect("commit error");
    warn!("save {} of {}", count, records.len());

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommitLog {
    id: Option<usize>,
    full_id: Option<String>,
    short_id: Option<String>,
    author_name: Option<String>,
    author_email: Option<String>,
    authored_date: Option<String>,
    committer_name: Option<String>,
    committer_email: Option<String>,
    committed_date: Option<String>,
    created_at: Option<String>,
    title: Option<String>,
    message: Option<String>,
    parent_ids: Option<Vec<String>>,
    additions: Option<usize>,
    deletions: Option<usize>,
}

impl CRUDEnable for CommitLog {
    type IdType = String;
}

impl CommitLog {
    pub fn from(id: usize, record: Record) -> Self {
        CommitLog {
            id: Some(id),
            full_id: Some(record.full_id),
            short_id: Some(record.short_id),
            author_name: Some(record.author_name),
            author_email: Some(record.author_email),
            authored_date: Some(record.authored_date),
            committer_name: Some(record.committer_name),
            committer_email: Some(record.committer_email),
            committed_date: Some(record.committed_date),
            created_at: Some(record.created_at),
            title: Some(record.title),
            message: Some(record.message),
            parent_ids: Some(record.parent_ids),
            additions: Some(record.stats.additions),
            deletions: Some(record.stats.deletions),
        }
    }
}

#[cfg(test)]
mod test {
    use fast_log::log::RuntimeType;

    use crate::store::sqlite::{save, CommitLog};

    #[async_std::test]
    pub async fn test_save() {
        fast_log::log::init_log("count.log", &RuntimeType::Std).unwrap();
        let commit = CommitLog {
            id: Some(706),
            full_id: Some("2".to_string()),
            short_id: Some("2".to_string()),
            author_name: Some("a".to_string()),
            author_email: Some("abc@mail.com".to_string()),
            authored_date: Some("20201212".to_string()),
            committer_name: Some("a".to_string()),
            committer_email: Some("abc@mail.com".to_string()),
            committed_date: Some("20201212".to_string()),
            created_at: Some("1.6".to_string()),
            title: Some("test a".to_string()),
            message: Some("test a message".to_string()),
            parent_ids: Some(vec!["1".to_string()]),
            additions: Some(10),
            deletions: Some(4),
        };

        let mut commit_new = commit.clone();
        commit_new.id = Some(707);

        let commits = vec![commit_new, commit];
        save(commits).await.expect("save fail");
    }
}
