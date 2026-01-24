//! TUI build runner that wraps make with a nice interface showing progress.
//!
//! Shows currently running tasks with spinners at the top, completed tasks below.
//! On success, cleans up and exits. On failure, prints error output after closing TUI.

use std::{
    collections::{HashMap, VecDeque},
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use clap::{Arg, ArgMatches};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
};
use sysinfo::System;
use tempfile::NamedTempFile;

use crate::utils;

const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
struct Task {
    name: String,
    status: TaskStatus,
    started_at: Instant,
    duration: Option<Duration>,
}

#[derive(Debug)]
enum BuildEvent {
    TaskStarted(String),
    TaskCompleted(String),
    TaskFailed(String),
    BuildFinished(i32),
    #[allow(dead_code)]
    Output(String),
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    MakeFailed {
        exit_code: i32,
        failed_tasks: Vec<String>,
    },
    FailedToFindRootDirectory,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<utils::FindRootDirectoryError> for Error {
    fn from(_: utils::FindRootDirectoryError) -> Self {
        Error::FailedToFindRootDirectory
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
    // Create temp log file
    let log_file = NamedTempFile::new()?;
    let log_path = log_file.path().to_path_buf();

    // Channel for build events
    let (tx, rx) = mpsc::channel();

    // Spawn make process
    // Remove RUSTUP_TOOLCHAIN to prevent nightly (used by tools crate) from propagating
    // to the build targets which need the default toolchain with thumbv4t-none-eabi
    let mut child = Command::new("make")
        .arg(format!("-j{jobs}"))
        .arg(format!("LOG_FILE={}", log_path.display()))
        .arg(target)
        .current_dir(utils::find_agb_root_directory()?)
        .env_remove("RUSTUP_TOOLCHAIN")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Spawn thread to read stdout
    let tx_stdout = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(event) = parse_line(&line) {
                let _ = tx_stdout.send(event);
            }
            let _ = tx_stdout.send(BuildEvent::Output(line));
        }
    });

    // Spawn thread to read stderr
    let tx_stderr = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx_stderr.send(BuildEvent::Output(line));
        }
    });

    // Spawn thread to wait for process completion
    let tx_done = tx;
    thread::spawn(move || {
        let status = child.wait().expect("Failed to wait on child");
        let _ = tx_done.send(BuildEvent::BuildFinished(status.code().unwrap_or(-1)));
    });

    // Run TUI
    let result = run_tui(rx);

    // Clean up log file on success, keep on failure for debugging
    match &result {
        Ok(()) => {
            // Log file automatically deleted when NamedTempFile drops
        }
        Err(Error::MakeFailed { failed_tasks, .. }) => {
            // Print failed task output from log - only sections with meaningful error content
            if !failed_tasks.is_empty()
                && let Ok(log_content) = std::fs::read_to_string(&log_path)
            {
                let sections_with_errors: Vec<_> = failed_tasks
                    .iter()
                    .filter_map(|task| {
                        extract_log_section(&log_content, task)
                            .filter(|s| has_meaningful_error_content(s))
                            .map(|s| (task.clone(), s))
                    })
                    .collect();

                if !sections_with_errors.is_empty() {
                    eprintln!("\n\x1b[31m=== Failed tasks ===\x1b[0m\n");
                    for (task, section) in sections_with_errors {
                        eprintln!("\x1b[33m=== {} ===\x1b[0m", task);
                        eprintln!("{}", section);
                    }
                }
            }

            // Persist the log file
            let _ = log_file.persist(&log_path);
        }
        Err(_) => {}
    }

    result
}

fn parse_line(line: &str) -> Option<BuildEvent> {
    // Look for our markers: ▶ (starting), ✓ (success), ✗ (failed)
    let line = line.trim();

    // Strip ANSI color codes for parsing
    let stripped = strip_ansi(line);

    #[allow(clippy::manual_map)]
    if let Some(starting) = stripped.strip_prefix("▶ ") {
        Some(BuildEvent::TaskStarted(starting.to_string()))
    } else if let Some(success) = stripped.strip_prefix("✓ ") {
        Some(BuildEvent::TaskCompleted(success.to_string()))
    } else if let Some(failed) = stripped.strip_prefix("✗ ") {
        Some(BuildEvent::TaskFailed(failed.to_string()))
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

fn extract_log_section(log: &str, task_name: &str) -> Option<String> {
    let marker = format!("=== {} ===", task_name);
    let start = log.find(&marker)?;
    let content_start = start + marker.len();

    // Find next section or end
    let next_section = log[content_start..]
        .find("\n===")
        .map(|i| content_start + i)
        .unwrap_or(log.len());

    Some(log[content_start..next_section].trim().to_string())
}

/// Check if a log section contains meaningful error content (not just noise)
fn has_meaningful_error_content(section: &str) -> bool {
    // Skip empty sections
    if section.trim().is_empty() {
        return false;
    }

    // Lines that are just build noise, not actual errors
    let noise_prefixes = [
        "Blocking waiting for file lock",
        "Compiling",
        "Finished",
        "Downloaded",
        "Downloading",
        "Fresh",
        "Building",
        "Running",
        "Updating",
    ];

    // Check if there's any line that isn't just noise
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_noise = noise_prefixes.iter().any(|p| trimmed.starts_with(p));
        if !is_noise {
            return true;
        }
    }

    false
}

const CPU_SAMPLE_COUNT: usize = 60;

fn run_tui(rx: Receiver<BuildEvent>) -> Result<(), Error> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut tasks: HashMap<String, Task> = HashMap::new();
    let mut exit_code: Option<i32> = None;
    let mut failed_tasks: Vec<String> = Vec::new();
    let start_time = Instant::now();

    // CPU monitoring
    let mut sys = System::new();
    let mut cpu_samples: VecDeque<u64> = VecDeque::with_capacity(CPU_SAMPLE_COUNT);
    let mut last_cpu_sample = Instant::now();

    loop {
        // Sample CPU usage periodically (every ~100ms)
        if last_cpu_sample.elapsed() >= Duration::from_millis(100) {
            sys.refresh_cpu_usage();
            let cpu_usage = sys.global_cpu_usage() as u64;
            if cpu_samples.len() >= CPU_SAMPLE_COUNT {
                cpu_samples.pop_front();
            }
            cpu_samples.push_back(cpu_usage);
            last_cpu_sample = Instant::now();
        }
        // Handle events from make
        while let Ok(event) = rx.try_recv() {
            match event {
                BuildEvent::TaskStarted(name) => {
                    tasks.insert(
                        name.clone(),
                        Task {
                            name,
                            status: TaskStatus::Running,
                            started_at: Instant::now(),
                            duration: None,
                        },
                    );
                }
                BuildEvent::TaskCompleted(name) => {
                    if let Some(task) = tasks.get_mut(&name) {
                        task.duration = Some(task.started_at.elapsed());
                        task.status = TaskStatus::Success;
                    }
                }
                BuildEvent::TaskFailed(name) => {
                    if let Some(task) = tasks.get_mut(&name) {
                        task.duration = Some(task.started_at.elapsed());
                        task.status = TaskStatus::Failed;
                    }
                    failed_tasks.push(name);
                }
                BuildEvent::BuildFinished(code) => {
                    exit_code = Some(code);
                }
                BuildEvent::Output(_) => {
                    // We don't display raw output in TUI, it's in the log file
                }
            }
        }

        // Check for quit or completion
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    break;
                }
                _ => {}
            }
        }

        // Draw UI
        terminal.draw(|frame| {
            draw_ui(frame, &tasks, start_time, exit_code, &cpu_samples);
        })?;

        // Exit if build finished
        if let Some(code) = exit_code {
            // Give a moment to see final state
            thread::sleep(Duration::from_millis(500));

            // Cleanup terminal
            disable_raw_mode()?;
            io::stdout().execute(LeaveAlternateScreen)?;

            if code == 0 {
                // Print completed tasks with durations
                let mut completed: Vec<_> = tasks
                    .values()
                    .filter(|t| t.status == TaskStatus::Success)
                    .collect();
                completed.sort_by(|a, b| a.name.cmp(&b.name));
                for task in completed {
                    let duration_str = task
                        .duration
                        .map(|d| format!(" ({:.1}s)", d.as_secs_f64()))
                        .unwrap_or_default();
                    println!("\x1b[32m✓\x1b[0m {}{}", task.name, duration_str);
                }
                println!(
                    "\n\x1b[32m✓ Build completed successfully ({:.1}s)\x1b[0m",
                    start_time.elapsed().as_secs_f64()
                );
                return Ok(());
            } else {
                return Err(Error::MakeFailed {
                    exit_code: code,
                    failed_tasks,
                });
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn draw_ui(
    frame: &mut Frame,
    tasks: &HashMap<String, Task>,
    start_time: Instant,
    exit_code: Option<i32>,
    cpu_samples: &VecDeque<u64>,
) {
    let elapsed = start_time.elapsed();
    let spinner_idx = (elapsed.as_millis() / 80) as usize % SPINNER_FRAMES.len();
    let spinner = SPINNER_FRAMES[spinner_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Running tasks + CPU
            Constraint::Min(10),   // Completed tasks
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

    // Header
    let status = match exit_code {
        None => format!("{} Building... ({:.1}s)", spinner, elapsed.as_secs_f64()),
        Some(0) => format!("✓ Build complete ({:.1}s)", elapsed.as_secs_f64()),
        Some(code) => format!(
            "✗ Build failed with exit code {} ({:.1}s)",
            code,
            elapsed.as_secs_f64()
        ),
    };
    let header =
        Paragraph::new(status).block(Block::default().borders(Borders::ALL).title("agb build"));
    frame.render_widget(header, chunks[0]);

    // Split running area: tasks on left, CPU graph on right
    let running_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),    // Running tasks list
            Constraint::Length(22), // CPU graph (fixed width)
        ])
        .split(chunks[1]);

    // Running tasks
    let running: Vec<ListItem> = tasks
        .values()
        .filter(|t| t.status == TaskStatus::Running)
        .map(|t| {
            let elapsed = t.started_at.elapsed();
            ListItem::new(format!(
                "{} {} ({:.1}s)",
                spinner,
                t.name,
                elapsed.as_secs_f64()
            ))
            .style(Style::default().fg(Color::Yellow))
        })
        .collect();

    let running_list =
        List::new(running).block(Block::default().borders(Borders::ALL).title(format!(
            "Running ({})",
            tasks.values().filter(|t| t.status == TaskStatus::Running).count()
        )));
    frame.render_widget(running_list, running_area[0]);

    // CPU usage sparkline
    let graph_width = running_area[1].width.saturating_sub(2) as usize; // -2 for borders
    let current_cpu = cpu_samples.back().copied().unwrap_or(0);

    // Take only the most recent samples that fit, pad with zeros on left if needed
    let recent_samples: Vec<u64> = cpu_samples
        .iter()
        .rev()
        .take(graph_width)
        .copied()
        .collect();
    let mut cpu_data: Vec<u64> = vec![0; graph_width.saturating_sub(recent_samples.len())];
    cpu_data.extend(recent_samples.into_iter().rev()); // Reverse back to chronological order

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("CPU {}%", current_cpu)),
        )
        .data(&cpu_data)
        .max(100)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(sparkline, running_area[1]);

    // Completed tasks (sorted by name)
    let mut completed_tasks: Vec<_> = tasks
        .values()
        .filter(|t| t.status != TaskStatus::Running)
        .collect();
    completed_tasks.sort_by(|a, b| a.name.cmp(&b.name));

    let completed: Vec<ListItem> = completed_tasks
        .into_iter()
        .map(|t| {
            let (symbol, color) = match t.status {
                TaskStatus::Success => ("✓", Color::Green),
                TaskStatus::Failed => ("✗", Color::Red),
                TaskStatus::Running => unreachable!(),
            };
            let duration_str = t
                .duration
                .map(|d| format!(" ({:.1}s)", d.as_secs_f64()))
                .unwrap_or_default();
            ListItem::new(format!("{} {}{}", symbol, t.name, duration_str))
                .style(Style::default().fg(color))
        })
        .collect();

    let success_count = tasks
        .values()
        .filter(|t| t.status == TaskStatus::Success)
        .count();
    let failed_count = tasks
        .values()
        .filter(|t| t.status == TaskStatus::Failed)
        .count();

    let completed_list = List::new(completed).block(Block::default().borders(Borders::ALL).title(
        format!("Completed ({} ✓, {} ✗)", success_count, failed_count),
    ));
    frame.render_widget(completed_list, chunks[2]);

    // Footer
    let footer =
        Paragraph::new("Press q or Ctrl+C to quit").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[3]);
}
