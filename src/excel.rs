//! อ่านข้อมูลนักเรียนและคะแนนจากไฟล์ Excel ของ SGS

use std::collections::HashMap;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

#[derive(Debug, Clone, PartialEq)]
pub struct Student {
    pub id: i64,
    pub name: String,
    /// คะแนนโดยใช้ชื่อ header ใน Excel เป็น key เช่น `S1`, `Midterm`, `Q1`
    pub scores: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub name: String,
    pub students: Vec<Student>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExcelData {
    pub rooms: Vec<Room>,
}

impl ExcelData {
    pub fn total_students(&self) -> usize {
        self.rooms.iter().map(|room| room.students.len()).sum()
    }

    /// รวมทุกชีตและเรียงตามรหัสนักเรียน เพื่อเทียบกับรายชื่อบนหน้า SGS ปัจจุบัน
    pub fn sorted_students(&self) -> Vec<&Student> {
        let mut students: Vec<_> = self
            .rooms
            .iter()
            .flat_map(|room| room.students.iter())
            .collect();
        students.sort_by_key(|student| student.id);
        students
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Before,
    After,
    Attribute,
    Study,
}

impl Page {
    pub fn thai(self) -> &'static str {
        match self {
            Self::Before => "ก่อนกลางภาค",
            Self::After => "หลังกลางภาค",
            Self::Attribute => "คุณลักษณะอันพึงประสงค์",
            Self::Study => "การอ่าน คิดวิเคราะห์ และเขียน",
        }
    }

    pub fn production_url(self) -> &'static str {
        match self {
            Self::Before => {
                "https://sgs.bopp-obec.info/sgs/TblTranscripts/Edit-TblTranscripts1-Table.aspx"
            }
            Self::After => {
                "https://sgs.bopp-obec.info/sgs/TblTranscripts/Edit-TblTranscripts2-Table.aspx"
            }
            Self::Attribute => {
                "https://sgs.bopp-obec.info/sgs/TblTranscriptsQ/Edit-TblTranscriptsQ-Table.aspx"
            }
            Self::Study => {
                "https://sgs.bopp-obec.info/sgs/TblTranscriptsL/Edit-TblTranscriptsL-Table.aspx"
            }
        }
    }

    pub fn mock_url(self) -> &'static str {
        match self {
            Self::Before => "http://localhost/sgs_tester/Edit-TblTranscripts1-Table.html",
            Self::After => "http://localhost/sgs_tester/Edit-TblTranscripts2-Table.html",
            Self::Attribute => "http://localhost/sgs_tester/Edit-TblTranscriptsQ-Table.html",
            Self::Study => "http://localhost/sgs_tester/Edit-TblTranscriptsL-Table.html",
        }
    }

    /// Repeater ของหน้าก่อนและหลังกลางภาคใช้ชื่อเดียวกันใน SGS
    pub fn repeater(self) -> &'static str {
        match self {
            Self::Before | Self::After => "TblTranscriptsTableControlRepeater",
            Self::Attribute => "TblTranscriptsQTableControlRepeater",
            Self::Study => "TblTranscriptsLTableControlRepeater",
        }
    }

    pub fn save_selector(self) -> &'static str {
        match self {
            Self::Before | Self::After => "input[id='ctl00_PageContent_TblTranscriptsSaveButton']",
            Self::Attribute => "input[id='ctl00_PageContent_TblTranscriptsQSaveButton']",
            Self::Study => "input[id='ctl00_PageContent_TblTranscriptsLSaveButton']",
        }
    }

    pub fn web_id_td(self) -> u8 {
        match self {
            Self::Before | Self::After => 4,
            Self::Attribute | Self::Study => 7,
        }
    }

    pub fn web_name_td(self) -> u8 {
        match self {
            Self::Before | Self::After => 5,
            Self::Attribute | Self::Study => 8,
        }
    }

    pub fn accepts_column(self, column: &str) -> bool {
        match self {
            Self::Before => {
                score_number(column, 'S').is_some_and(|number| number <= 9) || column == "Midterm"
            }
            Self::After => {
                score_number(column, 'S').is_some_and(|number| (10..=18).contains(&number))
                    || column == "Final"
            }
            Self::Attribute => {
                score_number(column, 'Q').is_some_and(|number| (1..=8).contains(&number))
            }
            Self::Study => {
                score_number(column, 'L').is_some_and(|number| (1..=5).contains(&number))
            }
        }
    }

    pub fn matches_url(self, url: &str) -> bool {
        let expected_file = match self {
            Self::Before => "Edit-TblTranscripts1-Table",
            Self::After => "Edit-TblTranscripts2-Table",
            Self::Attribute => "Edit-TblTranscriptsQ-Table",
            Self::Study => "Edit-TblTranscriptsL-Table",
        };
        url.contains(expected_file)
    }
}

fn score_number(column: &str, prefix: char) -> Option<u8> {
    column.strip_prefix(prefix)?.parse().ok()
}

#[derive(Debug, thiserror::Error)]
pub enum ExcelError {
    #[error("ไม่สามารถเปิดไฟล์ Excel: {0}")]
    Open(#[from] calamine::Error),
    #[error("ไฟล์ Excel ไม่มี sheet ที่อ่านได้")]
    NoSheets,
    #[error("sheet '{0}' มี header ไม่ถูกต้อง (ต้องขึ้นต้นด้วย stdID, student Name)")]
    BadHeader(String),
}

pub fn load_excel<P: AsRef<Path>>(path: P) -> Result<ExcelData, ExcelError> {
    let mut book = open_workbook_auto(path.as_ref())?;
    let sheet_names = book.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(ExcelError::NoSheets);
    }

    let mut rooms = Vec::with_capacity(sheet_names.len());
    for sheet_name in sheet_names {
        let range = book
            .worksheet_range(&sheet_name)
            .map_err(ExcelError::Open)?;
        let headers: Vec<String> = range
            .rows()
            .next()
            .map(|row| row.iter().map(cell_to_string).collect())
            .unwrap_or_default();

        if headers.first().map(String::as_str) != Some("stdID")
            || headers.get(1).map(String::as_str) != Some("student Name")
        {
            return Err(ExcelError::BadHeader(sheet_name));
        }

        let mut students = Vec::new();
        for row in range.rows().skip(1) {
            let Some(id) = row.first().and_then(cell_to_i64) else {
                continue;
            };
            let name = row.get(1).map(cell_to_string).unwrap_or_default();
            let scores = headers
                .iter()
                .enumerate()
                .skip(2)
                .filter_map(|(index, header)| {
                    row.get(index)
                        .and_then(cell_to_f64)
                        .map(|value| (header.clone(), value))
                })
                .collect();
            students.push(Student { id, name, scores });
        }
        students.sort_by_key(|student| student.id);
        rooms.push(Room {
            name: sheet_name,
            students,
        });
    }

    Ok(ExcelData { rooms })
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(value) => value.trim().to_owned(),
        Data::Float(value) if value.fract() == 0.0 => format!("{}", *value as i64),
        Data::Float(value) => value.to_string(),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Empty | Data::Error(_) => String::new(),
    }
}

fn cell_to_i64(cell: &Data) -> Option<i64> {
    match cell {
        Data::Int(value) => Some(*value),
        Data::Float(value) if value.fract() == 0.0 => Some(*value as i64),
        Data::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn cell_to_f64(cell: &Data) -> Option<f64> {
    match cell {
        Data::Int(value) => Some(*value as f64),
        Data::Float(value) => Some(*value),
        Data::String(value) if !value.trim().is_empty() => value.trim().parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sgs-all.xlsx")
    }

    #[test]
    fn loads_real_workbook_and_keeps_headers() {
        let path = fixture_path();
        if !path.exists() {
            return;
        }
        let data = load_excel(path).expect("โหลดไฟล์ตัวอย่างได้");
        assert_eq!(data.rooms.len(), 9);
        assert!(data.total_students() > 0);
        let first = &data.rooms[0].students[0];
        assert_eq!(first.id, 26623);
        assert_eq!(first.scores.get("S1"), Some(&30.0));
        assert_eq!(first.scores.get("Midterm"), Some(&15.0));
    }

    #[test]
    fn classifies_all_supported_score_columns() {
        for number in 1..=9 {
            assert!(Page::Before.accepts_column(&format!("S{number}")));
        }
        for number in 10..=18 {
            assert!(Page::After.accepts_column(&format!("S{number}")));
        }
        assert!(Page::Before.accepts_column("Midterm"));
        assert!(Page::After.accepts_column("Final"));
        assert!(Page::Attribute.accepts_column("Q8"));
        assert!(Page::Study.accepts_column("L5"));
        assert!(!Page::Before.accepts_column("S10"));
    }

    #[test]
    fn page_selectors_match_sgs_ids() {
        assert_eq!(Page::After.repeater(), "TblTranscriptsTableControlRepeater");
        assert!(Page::Attribute.save_selector().contains("QSaveButton"));
        assert!(Page::Study.matches_url(Page::Study.mock_url()));
        assert!(Page::Before.matches_url(Page::Before.production_url()));
    }
}
