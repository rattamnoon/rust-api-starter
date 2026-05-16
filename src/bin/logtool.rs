use std::{
    env, io,
    path::{Path, PathBuf},
};

use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand, ValueEnum};
use rust_api_starter::logging::{self, LogFileLevel};

#[derive(Parser)]
#[command(name = "logtool", about = "Inspect application log files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List,
    Tail {
        #[arg(long, value_enum)]
        level: LevelArg,
        #[arg(long, default_value_t = 20)]
        lines: usize,
    },
    Grep {
        #[arg(long, value_enum)]
        level: LevelArg,
        #[arg(long)]
        query: String,
        #[arg(long)]
        date: Option<String>,
    },
    Pretty {
        #[arg(long, value_enum)]
        level: LevelArg,
        #[arg(long)]
        date: Option<String>,
        #[arg(long, default_value_t = 20)]
        lines: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LevelArg {
    Info,
    Error,
}

impl From<LevelArg> for LogFileLevel {
    fn from(value: LevelArg) -> Self {
        match value {
            LevelArg::Info => LogFileLevel::Info,
            LevelArg::Error => LogFileLevel::Error,
        }
    }
}

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let log_dir = PathBuf::from(env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()));

    match cli.command {
        Command::List => list_logs(&log_dir)?,
        Command::Tail { level, lines } => tail_logs(&log_dir, level.into(), lines)?,
        Command::Grep { level, query, date } => grep_logs(&log_dir, level.into(), &query, date)?,
        Command::Pretty { level, date, lines } => pretty_logs(&log_dir, level.into(), date, lines)?,
    }

    Ok(())
}

fn list_logs(log_dir: &Path) -> io::Result<()> {
    let files = logging::list_log_files(log_dir)?;
    for file in files {
        println!(
            "{} {} {}",
            file.level.as_prefix(),
            file.date.format("%Y-%m-%d"),
            file.path.display()
        );
    }
    Ok(())
}

fn tail_logs(log_dir: &Path, level: LogFileLevel, lines: usize) -> io::Result<()> {
    let path = logging::today_log_path(log_dir, level);
    let log_lines = logging::read_log_lines(&path)?;
    for line in logging::tail_lines(&log_lines, lines) {
        println!("{line}");
    }
    Ok(())
}

fn grep_logs(
    log_dir: &Path,
    level: LogFileLevel,
    query: &str,
    date: Option<String>,
) -> io::Result<()> {
    let path = resolve_log_path(log_dir, level, date)?;
    let log_lines = logging::read_log_lines(&path)?;
    for line in logging::filter_lines(&log_lines, query) {
        println!("{line}");
    }
    Ok(())
}

fn pretty_logs(
    log_dir: &Path,
    level: LogFileLevel,
    date: Option<String>,
    lines: usize,
) -> io::Result<()> {
    let path = resolve_log_path(log_dir, level, date)?;
    let log_lines = logging::read_log_lines(&path)?;
    for line in logging::tail_lines(&log_lines, lines) {
        println!("{}", logging::pretty_format_line(&line));
    }
    Ok(())
}

fn resolve_log_path(
    log_dir: &Path,
    level: LogFileLevel,
    date: Option<String>,
) -> io::Result<PathBuf> {
    let date = match date {
        Some(value) => NaiveDate::parse_from_str(&value, "%Y-%m-%d")
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "date must be YYYY-MM-DD"))?,
        None => Local::now().date_naive(),
    };

    Ok(logging::log_file_path(log_dir, level, date))
}
