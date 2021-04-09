use log::info;
use rusqlite::{Connection, Statement};

use crate::config::ProjectId;
use crate::gitlab::CommitLogs;

pub fn init(conn: &Connection) -> Statement {
    conn.execute(
        "create table if not exists commit_log (
             project_id integer not null,
             full_commit_id text not null,
             short_commit_id text not null,
             author_name text not null,
             author_email text not null,
             authored_date text not null,
             committer_name text not null,
             committer_email text not null,
             committed_date text not null,
             created_at text not null,
             title text not null,
             message text not null,
             parent_ids text not null,
             additions integer not null,
             deletions integer not null,
             constraint commit_log_pk primary key (project_id, full_commit_id)
         )",
        [],
    )
    .expect("建表失败");
    conn.execute("delete from commit_log ", [])
        .expect("清空数据失败");

    let stmt = conn
        .prepare("insert into commit_log values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .expect("prepare insert statement 失败");
    stmt
}

pub fn insert(stmt: &mut Statement, logs: CommitLogs) {
    if logs.is_empty() {
        return;
    }

    let log = logs.get(0).unwrap();
    let id = ProjectId(log.project_id);
    info!("开始插入{}-{}条记录", id, logs.len());
    let start = std::time::Instant::now();

    for log in logs {
        stmt.execute([
            log.project_id.to_string(),
            log.full_id,
            log.short_id,
            log.author_name,
            log.author_email,
            log.authored_date,
            log.committer_name,
            log.committer_email,
            log.committed_date,
            log.created_at,
            log.title,
            log.message,
            log.parent_ids.join(","),
            log.stats.additions.to_string(),
            log.stats.deletions.to_string(),
        ])
        .expect("插入数据库失败");
    }

    info!("结束插入{},耗时{}ms", id, start.elapsed().as_millis());
}
