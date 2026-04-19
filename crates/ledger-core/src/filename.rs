use std::path::Path;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementFilename {
    pub vendor: String,
    pub account: String,
    pub year: u16,
    pub month: u8,
    pub doc_type: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FilenameError {
    #[error("filename must be UTF-8 and include extension")]
    InvalidUtf8OrExtension,
    #[error("expected format VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext")]
    InvalidFormat,
    #[error("year must be four digits")]
    InvalidYear,
    #[error("month must be 01..12")]
    InvalidMonth,
}

impl StatementFilename {
    pub fn parse(input: &str) -> Result<Self, FilenameError> {
        let stem = Path::new(input)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(FilenameError::InvalidUtf8OrExtension)?;

        let parts: Vec<&str> = stem.split("--").collect();
        if parts.len() != 4 {
            return Err(FilenameError::InvalidFormat);
        }

        let vendor = parts[0].trim();
        let account = parts[1].trim();
        let year_month = parts[2].trim();
        let doc_type = parts[3].trim();

        if vendor.is_empty() || account.is_empty() || doc_type.is_empty() {
            return Err(FilenameError::InvalidFormat);
        }

        let (year, month) = parse_year_month(year_month)?;

        Ok(Self {
            vendor: vendor.to_ascii_uppercase(),
            account: account.to_ascii_uppercase(),
            year,
            month,
            doc_type: doc_type.to_ascii_lowercase(),
        })
    }
}

fn parse_year_month(input: &str) -> Result<(u16, u8), FilenameError> {
    let mut iter = input.split('-');
    let year = iter.next().ok_or(FilenameError::InvalidFormat)?;
    let month = iter.next().ok_or(FilenameError::InvalidFormat)?;

    if iter.next().is_some() {
        return Err(FilenameError::InvalidFormat);
    }
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return Err(FilenameError::InvalidYear);
    }
    if month.len() != 2 || !month.chars().all(|c| c.is_ascii_digit()) {
        return Err(FilenameError::InvalidMonth);
    }

    let year_num = year
        .parse::<u16>()
        .map_err(|_| FilenameError::InvalidYear)?;
    let month_num = month
        .parse::<u8>()
        .map_err(|_| FilenameError::InvalidMonth)?;
    if !(1..=12).contains(&month_num) {
        return Err(FilenameError::InvalidMonth);
    }

    Ok((year_num, month_num))
}
