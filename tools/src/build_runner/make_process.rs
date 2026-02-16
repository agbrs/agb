use std::{
    io::{self, BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::utils;

#[derive(Debug)]
pub enum BuildEvent {
    TaskStarted(String),
    TaskCompleted(String),
    TaskFailed(String),
    BuildFinished(i32),
    #[allow(dead_code)]
    Output(String),
}

pub fn spawn(
    target: &str,
    jobs: usize,
    log_path: &Path,
) -> Result<Receiver<BuildEvent>, io::Error> {
    let (tx, rx) = mpsc::channel();

    // Remove RUSTUP_TOOLCHAIN to prevent the toolchain used by the tools crate
    // from propagating to build targets which need their own toolchain
    let mut child = Command::new("make")
        .arg(format!("-j{jobs}"))
        .arg(format!("LOG_FILE={}", log_path.display()))
        .arg(target)
        .current_dir(utils::find_agb_root_directory().map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "Failed to find agb root directory")
        })?)
        .env_remove("RUSTUP_TOOLCHAIN")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    spawn_output_reader(stdout, tx.clone(), true);
    spawn_output_reader(stderr, tx.clone(), false);

    let tx_done = tx;
    thread::spawn(move || {
        let status = child.wait().expect("Failed to wait on child");
        let _ = tx_done.send(BuildEvent::BuildFinished(status.code().unwrap_or(-1)));
    });

    Ok(rx)
}

fn spawn_output_reader<R: io::Read + Send + 'static>(
    reader: R,
    tx: Sender<BuildEvent>,
    parse_events: bool,
) {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            if parse_events && let Some(event) = parse_output(&line) {
                let _ = tx.send(event);
            }
            let _ = tx.send(BuildEvent::Output(line));
        }
    });
}

fn parse_output(line: &str) -> Option<BuildEvent> {
    let stripped = strip_ansi(line.trim());

    #[expect(clippy::manual_map)]
    if let Some(name) = stripped.strip_prefix("▶ ") {
        Some(BuildEvent::TaskStarted(name.to_string()))
    } else if let Some(name) = stripped.strip_prefix("✓ ") {
        Some(BuildEvent::TaskCompleted(name.to_string()))
    } else if let Some(name) = stripped.strip_prefix("✗ ") {
        Some(BuildEvent::TaskFailed(name.to_string()))
    } else {
        None
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Extract a task's log section from the combined log file.
/// The log file uses `=== task_name ===` as section markers.
pub fn extract_log_section(log: &str, task_name: &str) -> Option<String> {
    let marker = format!("=== {} ===", task_name);
    let start = log.find(&marker)?;
    let content_start = start + marker.len();

    let next_section = log[content_start..]
        .find("\n===")
        .map(|i| content_start + i)
        .unwrap_or(log.len());

    Some(log[content_start..next_section].trim().to_string())
}
