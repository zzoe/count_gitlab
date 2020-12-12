use anyhow::Result;
use rusqlite::Connection;
use serde_derive::{Deserialize, Serialize};
use simple_excel_writer::{Column, Row, Workbook};

use crate::config::ProjectID;
use std::fmt::{Display, Formatter};

pub fn create(conn: &Connection) {
    let mut wb = Workbook::create("/tmp.xlsx");
    let mut sheet = wb.create_sheet("SheetName");

    // set column width
    sheet.add_column(Column { width: 30.0 });
    sheet.add_column(Column { width: 30.0 });
    sheet.add_column(Column { width: 80.0 });
    sheet.add_column(Column { width: 60.0 });

    wb.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row(["Name", "Title", "Success", "XML Remark"]))?;
        sw.append_row(row([
            "Amy",
            "",
            "true",
            "<xml><tag>\"Hello\" & 'World'</tag></xml>",
        ]))?;
        sw.append_blank_rows(2);
        sw.append_row(row(["Tony", "", "", "retired"]))
    })
    .expect("write excel error!");

    let mut sheet = wb.create_sheet("Sheet2");
    wb.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row(["Name", "Title", "Success", "Remark"]))?;
        sw.append_row(row(["Amy", "Manager", "true", ""]))
    })
    .expect("write excel error!");

    wb.close().expect("close excel error!");
}

fn row(data: [&str; 4]) -> Row {
    let mut row = Row::new();
    for &cell in data.iter() {
        row.add_cell(cell);
    }
    row
}

struct Commits {
    project_id: ProjectID,
    author: Author,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Author(String);

impl Display for Author {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self.0 {
            "" => write!(f, "张三"),
            other => write!(f, "未识别的开发人员[{}]", other),
        }
    }
}

fn query(conn: &Connection) -> Result<Commits> {
    // conn.execute("", &[])?;
    unimplemented!()
}
