use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Local;
use log::{LevelFilter, Record, Metadata, Log};

pub struct FileLogger {
    log_dir: PathBuf,
}

impl FileLogger {
    pub fn new<P: AsRef<Path>>(log_dir: P) -> Self {
        fs::create_dir_all(&log_dir).ok();
        Self {
            log_dir: log_dir.as_ref().to_path_buf(),
        }
    }

    fn current_log_file(&self) -> PathBuf {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        self.log_dir.join(format!("storagerefresh-{}.log", date_str))
    }

    fn rotate_logs(&self) {
        // Keep only last 7 logs
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            let mut logs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with("storagerefresh-") && name.ends_with(".log")
                })
                .collect();

            // Sort by modified time
            logs.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

            if logs.len() > 7 {
                for log in logs.iter().take(logs.len() - 7) {
                    let _ = fs::remove_file(log.path());
                }
            }
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_file = self.current_log_file();
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "[{}] [{}] {}", timestamp, record.level(), record.args());
            }
            self.rotate_logs();

            // Also print to stdout for debugging
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logging<P: AsRef<Path>>(log_dir: P) -> anyhow::Result<()> {
    let logger = FileLogger::new(log_dir);
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| anyhow::anyhow!(e))
}
