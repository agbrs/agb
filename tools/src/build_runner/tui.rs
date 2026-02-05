use std::{
    cmp::Reverse,
    collections::HashMap,
    io,
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::{Error, cpu_monitor::CpuMonitor, make_process::BuildEvent};

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

struct State {
    tasks: HashMap<String, Task>,
    exit_code: Option<i32>,
    failed_tasks: Vec<String>,
    start_time: Instant,
    cpu_monitor: CpuMonitor,
}

impl State {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            exit_code: None,
            failed_tasks: Vec::new(),
            start_time: Instant::now(),
            cpu_monitor: CpuMonitor::new(),
        }
    }

    fn handle_event(&mut self, event: BuildEvent) {
        match event {
            BuildEvent::TaskStarted(name) => {
                self.tasks.insert(
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
                if let Some(task) = self.tasks.get_mut(&name) {
                    task.duration = Some(task.started_at.elapsed());
                    task.status = TaskStatus::Success;
                }
            }
            BuildEvent::TaskFailed(name) => {
                if let Some(task) = self.tasks.get_mut(&name) {
                    task.duration = Some(task.started_at.elapsed());
                    task.status = TaskStatus::Failed;
                }
                self.failed_tasks.push(name);
            }
            BuildEvent::BuildFinished(code) => {
                self.exit_code = Some(code);
            }
            BuildEvent::Output(_) => {}
        }
    }

    fn running_tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
    }

    fn completed_tasks_sorted(&self) -> Vec<&Task> {
        let mut tasks: Vec<_> = self
            .tasks
            .values()
            .filter(|t| t.status != TaskStatus::Running)
            .collect();
        tasks.sort_by_key(|t| Reverse(t.duration));
        tasks
    }

    fn success_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Success)
            .count()
    }

    fn failed_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Failed)
            .count()
    }
}

pub fn run(rx: Receiver<BuildEvent>) -> Result<(), Error> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = State::new();

    loop {
        state.cpu_monitor.update();

        while let Ok(event) = rx.try_recv() {
            state.handle_event(event);
        }

        if should_quit()? {
            break;
        }

        terminal.draw(|frame| draw_ui(frame, &state))?;

        if let Some(code) = state.exit_code {
            thread::sleep(Duration::from_millis(500));
            cleanup_terminal()?;
            return finish_build(&state, code);
        }
    }

    cleanup_terminal()?;
    Ok(())
}

fn should_quit() -> Result<bool, Error> {
    if event::poll(Duration::from_millis(50))?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        return Ok(matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            || (key.code == KeyCode::Char('c')
                && key.modifiers.contains(event::KeyModifiers::CONTROL)));
    }

    Ok(false)
}

fn cleanup_terminal() -> Result<(), Error> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn finish_build(state: &State, exit_code: i32) -> Result<(), Error> {
    if exit_code == 0 {
        print_success_summary(state);
        Ok(())
    } else {
        Err(Error::MakeFailed {
            exit_code,
            failed_tasks: state.failed_tasks.clone(),
        })
    }
}

fn print_success_summary(state: &State) {
    let mut completed: Vec<_> = state
        .tasks
        .values()
        .filter(|t| t.status == TaskStatus::Success)
        .collect();
    completed.sort_by_key(|t| Reverse(t.duration));

    for task in completed {
        let duration_str = task
            .duration
            .map(|d| format!(" ({:.1}s)", d.as_secs_f64()))
            .unwrap_or_default();
        println!("\x1b[32m✓\x1b[0m {}{}", task.name, duration_str);
    }

    println!(
        "\n\x1b[32m✓ Build completed successfully ({:.1}s)\x1b[0m",
        state.start_time.elapsed().as_secs_f64()
    );
}

fn draw_ui(frame: &mut Frame, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Running tasks + CPU
            Constraint::Min(10),   // Completed tasks
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

    draw_header(frame, state, chunks[0]);
    draw_running_section(frame, state, chunks[1]);
    draw_completed_section(frame, state, chunks[2]);
    draw_footer(frame, chunks[3]);
}

fn draw_header(frame: &mut Frame, state: &State, area: Rect) {
    let elapsed = state.start_time.elapsed();
    let status = match state.exit_code {
        None => {
            let spinner = current_spinner(&elapsed);
            format!("{} Building... ({:.1}s)", spinner, elapsed.as_secs_f64())
        }
        Some(0) => format!("✓ Build complete ({:.1}s)", elapsed.as_secs_f64()),
        Some(code) => format!(
            "✗ Build failed with exit code {} ({:.1}s)",
            code,
            elapsed.as_secs_f64()
        ),
    };

    let header =
        Paragraph::new(status).block(Block::default().borders(Borders::ALL).title("agb build"));
    frame.render_widget(header, area);
}

fn draw_running_section(frame: &mut Frame, state: &State, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),    // Running tasks list
            Constraint::Length(22), // CPU graph
        ])
        .split(area);

    draw_running_tasks(frame, state, chunks[0]);
    state.cpu_monitor.draw(frame, chunks[1]);
}

fn draw_running_tasks(frame: &mut Frame, state: &State, area: Rect) {
    let elapsed = state.start_time.elapsed();
    let spinner = current_spinner(&elapsed);

    let items: Vec<ListItem> = state
        .running_tasks()
        .map(|t| {
            let task_elapsed = t.started_at.elapsed();
            ListItem::new(format!(
                "{} {} ({:.1}s)",
                spinner,
                t.name,
                task_elapsed.as_secs_f64()
            ))
            .style(Style::default().fg(Color::Yellow))
        })
        .collect();

    let count = state.running_tasks().count();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Running ({})", count)),
    );
    frame.render_widget(list, area);
}

fn draw_completed_section(frame: &mut Frame, state: &State, area: Rect) {
    let items: Vec<ListItem> = state
        .completed_tasks_sorted()
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

    let title = format!(
        "Completed ({} ✓, {} ✗)",
        state.success_count(),
        state.failed_count()
    );
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer =
        Paragraph::new("Press q or Ctrl+C to quit").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

fn current_spinner(elapsed: &Duration) -> char {
    static SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    let idx = (elapsed.as_millis() / 80) as usize % SPINNER_FRAMES.len();
    SPINNER_FRAMES[idx]
}
