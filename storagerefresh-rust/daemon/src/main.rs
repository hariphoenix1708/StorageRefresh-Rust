mod conditions;
mod config;
mod env_detect;
mod logging;
mod maintenance;
mod scheduler;
mod state;
mod storage;

use clap::Parser;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Detect environment only and exit")]
    detect_only: bool,

    #[arg(long, help = "Check conditions and print pass/fail")]
    check_conditions: bool,

    #[arg(long, help = "Run once and exit")]
    once: bool,

    #[arg(long, help = "Dry run mode (overrides config)")]
    dry_run: bool,

    #[arg(long, help = "Config file path", default_value = "/data/adb/storagerefresh/config.toml")]
    config: String,

    #[arg(long, help = "State file path", default_value = "/data/adb/storagerefresh/state.json")]
    state: String,

    #[arg(long, help = "Log directory path", default_value = "/data/adb/storagerefresh/logs")]
    log_dir: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.detect_only {
        let env = env_detect::detect_environment();
        let storage = storage::detect_storage();

        println!("Environment:");
        println!("{}", serde_json::to_string_pretty(&env).unwrap());

        println!("Storage:");
        println!("{}", serde_json::to_string_pretty(&storage).unwrap());
        return Ok(());
    }

    let config = config::Config::load(&args.config)?;
    let mut state = state::AppState::load(&args.state);

    if state.detected_environment.is_none() {
        state.detected_environment = Some(env_detect::detect_environment());
    }

    if args.check_conditions {
        let result = conditions::check_all_conditions(
            config.battery.min_capacity_percent,
            config.battery.require_charging,
            config.battery.max_temperature_c,
            config.screen.min_idle_minutes,
            config.safety.require_foreground_check,
        );
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return Ok(());
    }

    logging::init_logging(&args.log_dir)?;

    if args.once {
        log::info!("Running in --once mode");
        run_cycle(&config, &mut state, &args.state, args.dry_run)?;
        return Ok(());
    }

    log::info!("Starting daemon with poll interval {} minutes", config.schedule.poll_interval_minutes);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        log::info!("Received exit signal, shutting down safely...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        if let Err(e) = run_cycle(&config, &mut state, &args.state, args.dry_run) {
            log::error!("Cycle error: {}", e);
        }

        // Sleep for the poll interval in small chunks to remain responsive to shutdown
        let poll_secs = config.schedule.poll_interval_minutes * 60;
        for _ in 0..poll_secs {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    log::info!("Daemon stopped");
    Ok(())
}

fn run_cycle(
    config: &config::Config,
    state: &mut state::AppState,
    state_path: &str,
    cli_dry_run: bool,
) -> anyhow::Result<()> {
    let result = conditions::check_all_conditions(
        config.battery.min_capacity_percent,
        config.battery.require_charging,
        config.battery.max_temperature_c,
        config.screen.min_idle_minutes,
        config.safety.require_foreground_check,
    );

    let dry_run = cli_dry_run || config.safety.dry_run;

    if scheduler::should_run(state.last_run_timestamp, config.schedule.min_interval_hours) {
        if result.all_met {
            log::info!("Conditions met, running maintenance");
            if let Err(e) = maintenance::run_maintenance(dry_run, config.storage.min_trim_len_mb) {
                log::error!("Maintenance failed: {}", e);
                state.last_run_result = format!("failed: {}", e);
            } else {
                log::info!("Maintenance successful");
                state.last_run_result = "success".to_string();
                state.last_run_timestamp = Utc::now().timestamp();
                state.run_count += 1;
            }
        } else {
            // Un-comment if you want to log every skipped poll, but it can be spammy
            // log::debug!("Conditions not met, skipping maintenance: {:?}", result);
            state.last_run_result = "skipped_conditions_not_met".to_string();
        }
    } else {
        // log::debug!("Minimum interval not elapsed, skipping maintenance");
        state.last_run_result = "skipped_interval_not_elapsed".to_string();
    }

    state.save(state_path)?;
    Ok(())
}
