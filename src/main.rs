mod audio;
mod duration;
mod timer;
mod ui;

use clap::Parser;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::{
    io::{self, IsTerminal, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use timer::{Status, Timer};

#[derive(Parser)]
#[command(
    version,
    about = "A terminal timer that keeps beeping until you stop it"
)]
struct Args {
    #[arg(help = "Seconds or a duration such as 5m or 1h30m10s")]
    duration: Option<String>,
    #[arg(long, help = "Use plain output instead of the terminal dashboard")]
    plain: bool,
    #[arg(short = 'v', long, help = "Use plain output and show the countdown")]
    verbose: bool,
}

pub struct App {
    pub timer: Option<Timer>,
    pub input: String,
    pub preset: usize,
    pub error: Option<String>,
    pub muted: bool,
    pub alarm: audio::Alarm,
}
impl App {
    fn new(duration: Option<Duration>) -> Self {
        Self {
            timer: duration.map(|d| Timer::new(d, Instant::now())),
            input: String::new(),
            preset: 1,
            error: None,
            muted: false,
            alarm: audio::Alarm::new(),
        }
    }
    fn key(&mut self, key: event::KeyEvent, now: Instant) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return true;
        }
        if let Some(timer) = &mut self.timer {
            match key.code {
                KeyCode::Char(' ') => timer.toggle_pause(now),
                KeyCode::Char('r') => {
                    timer.restart(now);
                    self.alarm.stop();
                }
                KeyCode::Char('m') => {
                    self.muted = !self.muted;
                    if self.muted {
                        self.alarm.stop();
                    }
                }
                KeyCode::Enter if timer.status == Status::Expired => {
                    self.alarm.stop();
                    self.timer = None;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Left | KeyCode::BackTab => self.preset = (self.preset + 2) % 3,
                KeyCode::Right | KeyCode::Tab => self.preset = (self.preset + 1) % 3,
                KeyCode::Backspace => {
                    self.input.pop();
                    self.error = None;
                }
                KeyCode::Enter => {
                    let input = if self.input.is_empty() {
                        ["5m", "15m", "25m"][self.preset]
                    } else {
                        &self.input
                    };
                    match duration::parse(input) {
                        Ok(d) => {
                            self.timer = Some(Timer::new(d, now));
                            self.input.clear();
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                KeyCode::Char('m') if self.input.is_empty() => self.muted = !self.muted,
                KeyCode::Char(c)
                    if (c.is_ascii_digit() || matches!(c, 'h' | 'm' | 's'))
                        && self.input.len() < 64 =>
                {
                    self.input.push(c);
                    self.error = None;
                }
                _ => {}
            }
        }
        false
    }
    fn tick(&mut self, now: Instant) -> io::Result<()> {
        if let Some(timer) = &mut self.timer {
            timer.tick(now);
        }
        let active = self
            .timer
            .as_ref()
            .is_some_and(|t| t.status == Status::Expired)
            && !self.muted;
        self.alarm.tick(active, now)
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tbeep: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let duration = args.duration.as_deref().map(duration::parse).transpose()?;
    let plain =
        args.plain || args.verbose || !io::stdin().is_terminal() || !io::stdout().is_terminal();
    if plain && duration.is_none() {
        return Err("A duration is required in plain mode. Try: tbeep --plain 5m".into());
    }
    let interrupted = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        signal.store(true, Ordering::Relaxed);
    })?;
    let mut app = App::new(duration);
    if plain {
        return run_plain(&mut app, args.verbose, &interrupted);
    }
    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(error) => {
            ratatui::restore();
            return Err(error.into());
        }
    };
    let result = run_tui(&mut terminal, &mut app, &interrupted);
    app.alarm.stop();
    ratatui::restore();
    result
}
fn run_tui(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    while !interrupted.load(Ordering::Relaxed) {
        let now = Instant::now();
        app.tick(now)?;
        terminal.draw(|frame| ui::draw(frame, app, now))?;
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && app.key(key, Instant::now())
        {
            break;
        }
    }
    Ok(())
}
fn run_plain(
    app: &mut App,
    verbose: bool,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Timer set for {} seconds...",
        app.timer.as_ref().unwrap().original.as_secs()
    );
    let mut last = String::new();
    let mut announced = false;
    let mut warned = false;
    while !interrupted.load(Ordering::Relaxed) {
        let now = Instant::now();
        app.tick(now)?;
        let timer = app.timer.as_ref().unwrap();
        if timer.status == Status::Expired && !announced {
            println!("\nTime's up! Beeping... (Ctrl+C to stop)");
            announced = true;
        } else if verbose && !announced {
            let remaining = duration::format(timer.remaining(now));
            if remaining != last {
                if io::stdout().is_terminal() {
                    print!("\rTime left: {remaining}   ");
                    io::stdout().flush()?;
                } else {
                    println!("Time left: {remaining}");
                }
                last = remaining;
            }
        }
        if !warned && let Some(warning) = &app.alarm.warning {
            eprintln!("{warning}");
            warned = true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    app.alarm.stop();
    println!(
        "\n{}",
        if announced {
            "Stopped."
        } else {
            "Timer cancelled."
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn press(app: &mut App, code: KeyCode, now: Instant) -> bool {
        app.key(event::KeyEvent::new(code, KeyModifiers::NONE), now)
    }
    #[test]
    fn entry_controls_and_validation() {
        let now = Instant::now();
        let mut app = App::new(None);
        press(&mut app, KeyCode::Right, now);
        press(&mut app, KeyCode::Enter, now);
        assert_eq!(
            app.timer.as_ref().unwrap().original,
            Duration::from_secs(1500)
        );
        app.timer = None;
        app.input = "0".into();
        press(&mut app, KeyCode::Enter, now);
        assert!(app.error.is_some());
        press(&mut app, KeyCode::Backspace, now);
        press(&mut app, KeyCode::Char('5'), now);
        press(&mut app, KeyCode::Char('m'), now);
        press(&mut app, KeyCode::Enter, now);
        assert_eq!(app.timer.unwrap().original, Duration::from_secs(300));
    }
    #[test]
    fn timer_keyboard_and_acknowledgement() {
        let now = Instant::now();
        let mut app = App::new(Some(Duration::from_secs(2)));
        app.timer = Some(Timer::new(Duration::from_secs(2), now));
        press(&mut app, KeyCode::Char(' '), now);
        assert_eq!(app.timer.as_ref().unwrap().status, Status::Paused);
        press(&mut app, KeyCode::Char(' '), now);
        press(&mut app, KeyCode::Char('r'), now);
        app.timer
            .as_mut()
            .unwrap()
            .tick(now + Duration::from_secs(3));
        press(&mut app, KeyCode::Char('m'), now);
        assert!(app.muted);
        press(&mut app, KeyCode::Enter, now);
        assert!(app.timer.is_none());
        assert!(press(&mut app, KeyCode::Esc, now));
    }
}
