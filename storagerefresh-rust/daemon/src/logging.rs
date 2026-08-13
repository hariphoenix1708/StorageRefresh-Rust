use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};

pub struct FileLogger {
    log_dir: PathBuf,
}

impl FileLogger {
    pub fn new<P: AsRef<Path>>(log_dir: P) -> Self {
        let log_dir = log_dir.as_ref().to_path_buf();
        fs::create_dir_all(&log_dir).ok();
        // World-writable dir so logs remain viewable by any shell user
        // regardless of the creating process's umask.
        let _ = fs::set_permissions(&log_dir, fs::Permissions::from_mode(0o777));
        let logger = Self { log_dir };
        logger.rotate_logs();
        logger
    }

    fn current_log_file(&self) -> PathBuf {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        self.log_dir
            .join(format!("storagerefresh-{}.log", date_str))
    }

    fn rotate_logs(&self) {
        // Keep only the last 7 daily logs.
        let Ok(entries) = fs::read_dir(&self.log_dir) else {
            return;
        };

        let mut logs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("storagerefresh-") && name.ends_with(".log")
            })
            .collect();

        logs.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        if logs.len() > 7 {
            for log in logs.iter().take(logs.len() - 7) {
                let _ = fs::remove_file(log.path());
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
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_file) {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(
                    file,
                    "[{}] [{}] {}",
                    timestamp,
                    record.level(),
                    record.args()
                );
                // Keep logs readable by any shell user.
                let _ = file.set_permissions(fs::Permissions::from_mode(0o666));
            }
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
