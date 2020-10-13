use anyhow::Result;
use rbatis::crud::{CRUDEnable, CRUD};
use rbatis::rbatis::Rbatis;
use serde::{Deserialize, Serialize};

use crate::statistic::Record;

pub async fn save(records: Vec<CommitLog>) -> Result<()> {
    let rb = Rbatis::new();
    rb.link("sqlite://gitlab.db").await?;

    let create_sql = "create table if not exists 'commit_log' (
    'id' number not null primary key,
    'full_id' text not null,
    'short_id' text not null,
    'author_name' text not null,
    'author_email' text not null,
    'authored_date' text not null,
    'committer_name' text not null,
    'committer_email' text not null,
    'committed_date' text not null,
    'created_at' text not null,
    'title' text not null,
    'message' text not null,
    'parent_ids' text not null,
    'additions' text not null,
    'deletions' text not null
)";

    rb.exec("", create_sql).await?;

    // for record in records{
    //     println!("{:#x?}", rb.save("", &record).await?);
    // }
    let r = rb.save_batch("", &records).await?;
    println!("{:#x?}", r);

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommitLog {
    id: usize,
    full_id: String,
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

impl CRUDEnable for CommitLog {
    type IdType = String;
}

impl CommitLog {
    pub fn from(id: usize, record: Record) -> Self {
        CommitLog {
            id,
            full_id: record.full_id,
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

#[cfg(test)]
mod test {
    use crate::store::sqlite::{save, CommitLog};
    use fast_log::log::RuntimeType;

    #[async_std::test]
    pub async fn test_save() {
        fast_log::log::init_log("rbatis.log", &RuntimeType::Std).unwrap();
        let commit = CommitLog {
            id: 706,
            full_id: "2".to_string(),
            short_id: "2".to_string(),
            author_name: "a".to_string(),
            author_email: "abc@mail.com".to_string(),
            authored_date: "20201212".to_string(),
            committer_name: "a".to_string(),
            committer_email: "abc@mail.com".to_string(),
            committed_date: "20201212".to_string(),
            created_at: "1.6".to_string(),
            title: "test a".to_string(),
            message: "test a message".to_string(),
            parent_ids: vec!["1".to_string()],
            additions: 10,
            deletions: 4,
        };

        let commits = vec![commit.clone(), commit];
        save(commits).await.expect("save fail");
    }
}
