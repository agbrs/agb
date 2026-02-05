//! TUI build runner that wraps make with a nice interface showing progress.
//!
//! Shows currently running tasks with spinners at the top, completed tasks below.
//! On success, cleans up and exits. On failure, prints error output after closing TUI.

mod cpu_monitor;
mod make_process;
mod tui;

use std::{io, thread};

use clap::{Arg, ArgMatches};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    MakeFailed {
        exit_code: i32,
        failed_tasks: Vec<String>,
    },
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn command() -> clap::Command {
    clap::Command::new("build")
        .about("Run make with a TUI showing build progress")
        .arg(
            Arg::new("target")
                .help("Make target to build (default: ci)")
                .default_value("ci"),
        )
        .arg(
            Arg::new("jobs")
                .short('j')
                .long("jobs")
                .help("Number of parallel jobs")
                .default_value("0"),
        )
}

pub fn build(matches: &ArgMatches) -> Result<(), Error> {
    let target = matches.get_one::<String>("target").unwrap();
    let jobs: usize = matches
        .get_one::<String>("jobs")
        .unwrap()
        .parse()
        .unwrap_or(0);

    let jobs = if jobs == 0 {
        thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    } else {
        jobs
    };

    run_build(target, jobs)
}

fn run_build(target: &str, jobs: usize) -> Result<(), Error> {
    let log_file = NamedTempFile::new()?;
    let log_path = log_file.path().to_path_buf();

    let rx = make_process::spawn(target, jobs, &log_path)?;
    let result = tui::run(rx);

    handle_build_result(&result, log_file, &log_path);
    result
}

fn handle_build_result(
    result: &Result<(), Error>,
    log_file: NamedTempFile,
    log_path: &std::path::Path,
) {
    if let Err(Error::MakeFailed { failed_tasks, .. }) = result {
        if !failed_tasks.is_empty()
            && let Ok(log_content) = std::fs::read_to_string(log_path)
        {
            print_failed_task_logs(&log_content, failed_tasks);
        }
        let _ = log_file.persist(log_path);
    }
}

fn print_failed_task_logs(log_content: &str, failed_tasks: &[String]) {
    let sections: Vec<_> = failed_tasks
        .iter()
        .filter_map(|task| {
            make_process::extract_log_section(log_content, task).map(|s| (task.clone(), s))
        })
        .collect();

    if !sections.is_empty() {
        eprintln!("\n\x1b[31m=== Failed tasks ===\x1b[0m\n");
        for (task, section) in sections {
            eprintln!("\x1b[33m=== {} ===\x1b[0m", task);
            eprintln!("{}", section);
        }
    }
}
