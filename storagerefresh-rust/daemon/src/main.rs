mod conditions;
mod config;
mod env_detect;
mod logging;
mod maintenance;
mod scheduler;
mod state;
mod storage;

use chrono::Utc;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Detect environment only and exit")]
    detect_only: bool,

    #[arg(long, help = "Print current conditions and exit")]
    check_conditions: bool,

    #[arg(
        long,
        help = "Print a full status snapshot (state + conditions) and exit"
    )]
    status: bool,

    #[arg(long, help = "Run one maintenance cycle and exit")]
    once: bool,

    #[arg(long, help = "Dry run mode (overrides config)")]
    dry_run: bool,

    #[arg(
        long,
        help = "Config file path",
        default_value = "/data/adb/storagerefresh/config.toml"
    )]
    config: String,

    #[arg(
        long,
        help = "State file path",
        default_value = "/data/adb/storagerefresh/state.json"
    )]
    state: String,

    #[arg(
        long,
        help = "Log directory path",
        default_value = "/data/local/tmp/StorageRefresh"
    )]
    log_dir: String,

    #[arg(
        long,
        help = "PID file path",
        default_value = "/data/adb/storagerefresh/daemon.pid"
    )]
    pidfile: String,
}

/// Shared shutdown flag, set by both SIGINT (Ctrl-C) and SIGTERM.
static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_signal(_: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn install_signal_handlers() {
    // SAFETY: the handler only stores to an AtomicBool, which is
    // async-signal-safe.
    unsafe {
        let handler = handle_signal as *const () as libc::sighandler_t;
        libc::signal(libc::SIGINT, handler);
        libc::signal(libc::SIGTERM, handler);
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.detect_only {
        let env = env_detect::detect_environment();
        let storage = storage::detect_storage();

        println!("Environment:");
        println!("{}", serde_json::to_string_pretty(&env)?);

        println!("Storage:");
        println!("{}", serde_json::to_string_pretty(&storage)?);
        return Ok(());
    }

    let config = config::Config::load(&args.config)?;
    let mut state = state::AppState::load(&args.state);

    if state.detected_environment.is_none() {
        state.detected_environment = Some(env_detect::detect_environment());
    }

    if args.check_conditions {
        let result = conditions::check_all_conditions(&config, &state);
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if args.status {
        print_status(&config, &state)?;
        return Ok(());
    }

    logging::init_logging(&args.log_dir)?;

    if args.once {
        log::info!("Running in --once mode");
        run_cycle(&config, &mut state, &args.state, args.dry_run)?;
        print_status(&config, &state)?;
        return Ok(());
    }

    // Daemon mode
    install_signal_handlers();
    write_pidfile(&args.pidfile)?;
    log::info!(
        "Starting daemon with poll interval {} minutes",
        config.schedule.poll_interval_minutes
    );

    while RUNNING.load(Ordering::SeqCst) {
        if let Err(e) = run_cycle(&config, &mut state, &args.state, args.dry_run) {
            log::error!("Cycle error: {}", e);
        }

        // Sleep for the poll interval in small chunks to remain responsive
        // to shutdown signals.
        let poll_secs = config.schedule.poll_interval_minutes * 60;
        for _ in 0..poll_secs {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    log::info!("Daemon stopped");
    let _ = fs::remove_file(&args.pidfile);
    Ok(())
}

fn write_pidfile(path: &str) -> anyhow::Result<()> {
    let pidfile = PathBuf::from(path);
    if let Some(parent) = pidfile.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&pidfile, format!("{}\n", std::process::id()))?;
    Ok(())
}

fn run_cycle(
    config: &config::Config,
    state: &mut state::AppState,
    state_path: &str,
    cli_dry_run: bool,
) -> anyhow::Result<()> {
    // Track how long the screen has been off so `min_idle_minutes` is honored.
    match conditions::screen::screen_state() {
        Some(false) => {
            if state.screen_off_since.is_none() {
                state.screen_off_since = Some(Utc::now().timestamp());
            }
        }
        _ => state.screen_off_since = None,
    }

    let result = conditions::check_all_conditions(config, state);

    let dry_run = cli_dry_run || config.safety.dry_run;

    if scheduler::should_run(state.last_run_timestamp, config.schedule.min_interval_hours) {
        if result.all_met {
            log::info!("Conditions met, running maintenance");
            match maintenance::run_maintenance(dry_run, &config.storage) {
                Ok(trimmed) => {
                    log::info!("Maintenance successful ({} bytes trimmed)", trimmed);
                    state.last_run_result = "success".to_string();
                    state.last_trimmed_bytes = trimmed;
                    state.last_run_timestamp = Utc::now().timestamp();
                    state.run_count += 1;
                }
                Err(e) => {
                    log::error!("Maintenance failed: {}", e);
                    state.last_run_result = format!("failed: {}", e);
                }
            }
        } else {
            log::debug!(
                "Conditions not met, skipping maintenance: battery={} screen={} idle={} foreground={}",
                result.battery_ok,
                result.screen_ok,
                result.idle_ok,
                result.foreground_ok
            );
            state.last_run_result = "skipped_conditions_not_met".to_string();
        }
    } else {
        log::debug!("Minimum interval not elapsed, skipping maintenance");
        state.last_run_result = "skipped_interval_not_elapsed".to_string();
    }

    state.save(state_path)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct StatusOutput {
    environment: env_detect::Environment,
    storage: Vec<storage::StorageInfo>,
    state: state::AppState,
    conditions: conditions::ConditionsResult,
    running: bool,
    pid: Option<u32>,
}

fn print_status(config: &config::Config, state: &state::AppState) -> anyhow::Result<()> {
    let output = StatusOutput {
        environment: state.detected_environment.clone().unwrap_or_default(),
        storage: storage::detect_storage(),
        state: state.clone(),
        conditions: conditions::check_all_conditions(config, state),
        running: daemon_running(),
        pid: read_daemon_pid(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn daemon_running() -> bool {
    let Some(pid) = read_daemon_pid() else {
        return false;
    };
    let path = format!("/proc/{}/cmdline", pid);
    fs::read_to_string(path)
        .map(|c| c.contains("storagerefresh-rust"))
        .unwrap_or(false)
}

fn read_daemon_pid() -> Option<u32> {
    fs::read_to_string("/data/adb/storagerefresh/daemon.pid")
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
