use std::collections::HashMap;
use std::ops::Add;

use anyhow::Result;
use chrono::{Date, Datelike, Duration, Local, NaiveDate, TimeZone, Weekday};
use log::{debug, info};
use rusqlite::{Connection, Statement};
use simple_excel_writer::{Row, SheetWriter, Workbook};

use crate::config::{Author, ProjectId};
use crate::CONFIG;

const SQL: &str = "SELECT c.project_id, c.author_name, count(1) as 'commit_times', sum(c.additions) as 'additions' \
    FROM commit_log c \
    where c.author_email = c.committer_email \
        and instr(c.parent_ids, ',') = 0 \
        and c.authored_date >= :start \
        and c.authored_date < :end \
    group by c.project_id, c.author_name \
    order by c.author_name,c.project_id";

pub fn create() -> Result<()> {
    let [week, month] = query()?;

    gen_file(week, "周报")?;
    gen_file(month, "月报")?;

    Ok(())
}

//单账号单项目
#[derive(Debug)]
struct RecordPerAccount {
    project_id: ProjectId,
    account: String,
    commit_times: u32,
    additions: u32,
}

//单人单项目
#[derive(Clone, Debug, Default)]
struct RecordPerAuthor {
    projects: Vec<ProjectId>,
    accounts: Vec<String>,
    commit_times: u32,
    additions: u32,
    additions_per_day: u32,
}

impl From<RecordPerAuthor> for Row {
    fn from(records: RecordPerAuthor) -> Self {
        let mut row = Row::new();
        row.add_cell(
            records
                .projects
                .iter()
                .fold(String::new(), |i, p| format!("{}{}\n", i, p))
                .trim(),
        );
        row.add_cell(
            records
                .accounts
                .iter()
                .fold(String::new(), |i, a| format!("{}{}\n", i, a))
                .trim(),
        );
        row.add_cell(Author(records.accounts.get(0).unwrap().clone()).to_string());
        row.add_cell(records.commit_times as f64);
        row.add_cell(records.additions as f64);
        row.add_cell(records.additions_per_day as f64);
        row
    }
}

trait Report {
    fn add(&mut self, record: RecordPerAccount);
}

impl Report for HashMap<Author, RecordPerAuthor> {
    fn add(&mut self, record: RecordPerAccount) {
        let author = Author(record.account.clone());
        match self.get_mut(&author) {
            Some(record_per_author) => {
                if !record_per_author.projects.contains(&record.project_id) {
                    record_per_author.projects.push(record.project_id);
                }
                if !record_per_author.accounts.contains(&record.account) {
                    record_per_author.accounts.push(record.account);
                }
                record_per_author.commit_times += record.commit_times;
                record_per_author.additions += record.additions;
            }
            None => {
                let new_record = RecordPerAuthor {
                    projects: vec![record.project_id],
                    accounts: vec![record.account],
                    commit_times: record.commit_times,
                    additions: record.additions,
                    additions_per_day: 0,
                };
                self.insert(author, new_record);
            }
        }
    }
}

fn query() -> Result<[HashMap<Author, RecordPerAuthor>; 2]> {
    let today = chrono::Local::today();
    let conn = Connection::open(&*CONFIG.sqlite)?;
    let mut stmt = conn.prepare(SQL)?;

    // 周报
    let mut week_report = HashMap::new();
    if today.weekday().eq(&Weekday::Mon) {
        let start = today.add(Duration::days(-7));
        gen_report(&mut stmt, &mut week_report, start, today)?;
    }

    // 月报
    let mut month_report = HashMap::new();
    if today.day().eq(&1) {
        let mut start_year = today.year();
        let mut start_month = today.month() - 1;
        if start_month == 0 {
            start_year -= 1;
            start_month = 12;
        }
        let start = NaiveDate::from_ymd(start_year, start_month, 1);

        gen_report(
            &mut stmt,
            &mut month_report,
            Local.from_local_date(&start).unwrap(),
            today,
        )?;
    }

    Ok([week_report, month_report])
}

fn gen_report(
    stmt: &mut Statement,
    report: &mut HashMap<Author, RecordPerAuthor>,
    start: Date<Local>,
    end: Date<Local>,
) -> Result<()> {
    let days = end.signed_duration_since(start).num_days() as u32;
    info!("days: {} [{} - {}]", days, start, end);
    let records = stmt.query_map(
        &[
            (":start", &start.format("%F").to_string()),
            (":end", &end.format("%F").to_string()),
        ],
        |row| {
            Ok(RecordPerAccount {
                project_id: ProjectId(row.get(0)?),
                account: row.get(1)?,
                commit_times: row.get(2)?,
                additions: row.get(3)?,
            })
        },
    )?;

    for record_res in records {
        let record = record_res?;
        debug!("{:?}", record);
        report.add(record);
    }

    for record in report.values_mut() {
        record.additions_per_day = record.additions / days;
    }

    Ok(())
}

fn gen_file(report: HashMap<Author, RecordPerAuthor>, file_name: &str) -> Result<()> {
    if report.is_empty() {
        return Ok(());
    }

    let file_name = &*format!(
        "{}-{}.xlsx",
        file_name,
        chrono::Local::today().format("%Y%m%d")
    );
    let mut wb = Workbook::create(file_name);
    let mut sheet = wb.create_sheet("代码量统计");
    wb.write_sheet(&mut sheet, |sheet_writer| {
        add_title(sheet_writer)?;
        for (_, record) in report {
            sheet_writer.append_row(record.into())?;
        }
        Ok(())
    })?;
    wb.close()?;
    info!("生成文件：{}", file_name);

    Ok(())
}

fn add_title(sheet_writer: &mut SheetWriter) -> std::io::Result<()> {
    let mut title = Row::new();
    title.add_cell("项目");
    title.add_cell("账号");
    title.add_cell("姓名");
    title.add_cell("提交次数");
    title.add_cell("新增代码总行数");
    title.add_cell("日均新增代码行数");

    sheet_writer.append_row(title)
}
