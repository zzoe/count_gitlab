use anyhow::Result;
use log::info;
use rusqlite::{Connection, Statement, NO_PARAMS};

use crate::gitlab::CommitLogs;
use crate::CONFIG;

pub fn connect() -> Result<Connection> {
    let conn = Connection::open(&*CONFIG.sqlite)?;
    info!("{}", conn.is_autocommit());

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
        NO_PARAMS,
    )?;

    Ok(conn)
}

pub fn prepare_insert(conn: &Connection) -> Result<Statement> {
    Ok(conn.prepare("insert into commit_log values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")?)
}

pub fn insert(stmt: &mut Statement, logs: CommitLogs) -> Result<()> {
    for log in logs {
        stmt.execute(&[
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
        ])?;
    }
    Ok(())
}
