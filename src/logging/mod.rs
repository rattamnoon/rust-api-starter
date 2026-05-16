use std::{
    cmp::Reverse,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use chrono::{Local, NaiveDate};
use serde_json::Value;
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, Layer,
    filter::LevelFilter,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::config::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFileLevel {
    Info,
    Error,
}

impl LogFileLevel {
    pub const fn as_prefix(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogFileInfo {
    pub level: LogFileLevel,
    pub date: NaiveDate,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ParsedLogLine {
    pub raw: String,
    pub json: Value,
}

#[derive(Clone)]
struct DailyFileWriter {
    log_dir: PathBuf,
    level: LogFileLevel,
}

impl DailyFileWriter {
    fn new(log_dir: PathBuf, level: LogFileLevel) -> Self {
        Self { log_dir, level }
    }
}

impl<'a> MakeWriter<'a> for DailyFileWriter {
    type Writer = File;

    fn make_writer(&'a self) -> Self::Writer {
        let path = today_log_path(&self.log_dir, self.level);
        open_log_file(&path)
            .unwrap_or_else(|error| panic!("failed to open log file {}: {error}", path.display()))
    }
}

pub fn init(settings: &Settings) -> io::Result<()> {
    let log_dir = PathBuf::from(&settings.log_dir);
    ensure_log_dir(&log_dir)?;
    let _ = LogTracer::init();

    let stdout_filter = EnvFilter::try_new(settings.rust_log.clone())
        .unwrap_or_else(|_| EnvFilter::new("info,actix_web=info"));

    let stdout_layer = fmt::layer().with_target(true).with_filter(stdout_filter);

    let info_file_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_writer(DailyFileWriter::new(log_dir.clone(), LogFileLevel::Info))
        .with_filter(LevelFilter::INFO);

    let error_file_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_writer(DailyFileWriter::new(log_dir, LogFileLevel::Error))
        .with_filter(LevelFilter::ERROR);

    let _ = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(info_file_layer)
        .with(error_file_layer)
        .try_init();

    Ok(())
}

pub fn ensure_log_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub fn log_file_path(log_dir: &Path, level: LogFileLevel, date: NaiveDate) -> PathBuf {
    log_dir.join(format!(
        "{}_{}.log",
        level.as_prefix(),
        date.format("%Y-%m-%d")
    ))
}

pub fn today_log_path(log_dir: &Path, level: LogFileLevel) -> PathBuf {
    log_file_path(log_dir, level, Local::now().date_naive())
}

pub fn open_log_file(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    OpenOptions::new().create(true).append(true).open(path)
}

pub fn list_log_files(log_dir: &Path) -> io::Result<Vec<LogFileInfo>> {
    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(info) = parse_log_file_info(&path) {
            files.push(info);
        }
    }

    files.sort_by_key(|info| (Reverse(info.date), level_rank(info.level)));
    Ok(files)
}

pub fn read_log_lines(path: &Path) -> io::Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn tail_lines(lines: &[String], count: usize) -> Vec<String> {
    let start = lines.len().saturating_sub(count);
    lines[start..].to_vec()
}

pub fn filter_lines<'a>(lines: &'a [String], query: &str) -> Vec<&'a str> {
    lines
        .iter()
        .filter(|line| line.contains(query))
        .map(String::as_str)
        .collect()
}

pub fn parse_json_line(line: &str) -> Result<ParsedLogLine, serde_json::Error> {
    Ok(ParsedLogLine {
        raw: line.to_string(),
        json: serde_json::from_str(line)?,
    })
}

pub fn pretty_format_line(line: &str) -> String {
    match parse_json_line(line) {
        Ok(parsed) => {
            let timestamp = parsed
                .json
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("-");
            let level = parsed
                .json
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("-");
            let target = parsed
                .json
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("-");
            let message = parsed
                .json
                .get("fields")
                .and_then(Value::as_object)
                .and_then(|fields| fields.get("message"))
                .and_then(Value::as_str)
                .unwrap_or(parsed.raw.as_str());

            format!("[{timestamp}] {level:<5} {target} {message}")
        }
        Err(_) => line.to_string(),
    }
}

pub fn append_test_line(path: &Path, line: &str) -> io::Result<()> {
    let mut file = open_log_file(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn level_from_str(value: &str) -> Option<LogFileLevel> {
    match value.to_ascii_lowercase().as_str() {
        "info" => Some(LogFileLevel::Info),
        "error" => Some(LogFileLevel::Error),
        _ => None,
    }
}

fn parse_log_file_info(path: &Path) -> Option<LogFileInfo> {
    let file_name = path.file_name()?.to_str()?;
    let (level, date_str) = if let Some(date_str) = file_name
        .strip_prefix("INFO_")
        .and_then(|rest| rest.strip_suffix(".log"))
    {
        (LogFileLevel::Info, date_str)
    } else if let Some(date_str) = file_name
        .strip_prefix("ERROR_")
        .and_then(|rest| rest.strip_suffix(".log"))
    {
        (LogFileLevel::Error, date_str)
    } else {
        return None;
    };

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    Some(LogFileInfo {
        level,
        date,
        path: path.to_path_buf(),
    })
}

fn level_rank(level: LogFileLevel) -> u8 {
    match level {
        LogFileLevel::Error => 0,
        LogFileLevel::Info => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("rust-api-starter-{label}-{unique}"))
    }

    #[test]
    fn builds_expected_paths() {
        let dir = PathBuf::from("./logs");
        let date = NaiveDate::from_ymd_opt(2026, 5, 16).expect("date should be valid");

        assert_eq!(
            log_file_path(&dir, LogFileLevel::Info, date),
            PathBuf::from("./logs/INFO_2026-05-16.log")
        );
        assert_eq!(
            log_file_path(&dir, LogFileLevel::Error, date),
            PathBuf::from("./logs/ERROR_2026-05-16.log")
        );
    }

    #[test]
    fn creates_missing_log_directory() {
        let dir = temp_dir("ensure-dir");
        ensure_log_dir(&dir).expect("directory should be created");
        assert!(dir.exists());
        fs::remove_dir_all(dir).expect("cleanup should succeed");
    }

    #[test]
    fn parses_json_and_pretty_formats() {
        let line = r#"{"timestamp":"2026-05-16T10:00:00Z","level":"INFO","target":"api","fields":{"message":"hello world"}}"#;
        let parsed = parse_json_line(line).expect("json should parse");

        assert_eq!(parsed.json["level"], "INFO");
        assert!(pretty_format_line(line).contains("hello world"));
    }

    #[test]
    fn lists_and_filters_logs() {
        let dir = temp_dir("list-logs");
        ensure_log_dir(&dir).expect("directory should be created");

        let info_path = dir.join("INFO_2026-05-16.log");
        let error_path = dir.join("ERROR_2026-05-16.log");
        append_test_line(&info_path, r#"{"fields":{"message":"auth ok"}}"#)
            .expect("should write info log");
        append_test_line(&error_path, r#"{"fields":{"message":"db error"}}"#)
            .expect("should write error log");

        let files = list_log_files(&dir).expect("files should list");
        assert_eq!(files.len(), 2);

        let error_lines = read_log_lines(&error_path).expect("error lines should read");
        let filtered = filter_lines(&error_lines, "db error");
        assert_eq!(filtered.len(), 1);

        fs::remove_dir_all(dir).expect("cleanup should succeed");
    }
}
