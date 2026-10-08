use crate::{App, duration, timer::Status};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Wrap},
};
use std::time::Instant;

const CYAN: Color = Color::Rgb(80, 220, 230);
const DIM: Color = Color::Rgb(130, 145, 165);
const DIGITS: [[&str; 5]; 10] = [
    ["███", "█ █", "█ █", "█ █", "███"],
    [" █ ", "██ ", " █ ", " █ ", "███"],
    ["███", "  █", "███", "█  ", "███"],
    ["███", "  █", "███", "  █", "███"],
    ["█ █", "█ █", "███", "  █", "  █"],
    ["███", "█  ", "███", "  █", "███"],
    ["███", "█  ", "███", "█ █", "███"],
    ["███", "  █", "  █", "  █", "  █"],
    ["███", "█ █", "███", "█ █", "███"],
    ["███", "█ █", "███", "  █", "███"],
];
fn large_digits(value: &str) -> Vec<Line<'static>> {
    (0..5)
        .map(|row| {
            Line::from(
                value
                    .chars()
                    .map(|c| {
                        if let Some(n) = c.to_digit(10) {
                            DIGITS[n as usize][row]
                        } else if row == 1 || row == 3 {
                            " • "
                        } else {
                            "   "
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .collect()
}
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

pub fn draw(frame: &mut Frame, app: &App, now: Instant) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Reset)),
        area,
    );
    let panel = centered(area, 76, 24);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CYAN))
        .title(" tbeep ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(panel);
    frame.render_widget(Clear, panel);
    frame.render_widget(block, panel);
    if area.width < 28 || area.height < 8 {
        let value = app
            .timer
            .as_ref()
            .map(|t| duration::format(t.remaining(now)))
            .unwrap_or_else(|| "Set duration".into());
        frame.render_widget(
            Paragraph::new(format!("{value}\nEnlarge terminal\nq / Esc: quit"))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    }
    let big = inner.width >= 48 && inner.height >= 17;
    let sections = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(if big { 7 } else { 3 }),
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(4),
    ])
    .split(inner);
    let sound = if app.muted { "MUTED" } else { "SOUND ON" };
    let status = app
        .timer
        .as_ref()
        .map(|t| match t.status {
            Status::Running => "RUNNING",
            Status::Paused => "PAUSED",
            Status::Expired => "TIME'S UP",
        })
        .unwrap_or("SET TIMER");
    let accent = if app
        .timer
        .as_ref()
        .is_some_and(|t| t.status == Status::Expired)
    {
        Color::Yellow
    } else {
        CYAN
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                status,
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("   /   {sound}"), Style::default().fg(DIM)),
        ]))
        .alignment(Alignment::Center),
        sections[0],
    );
    if inner.height < 12 || inner.width < 36 {
        let mut lines = vec![Line::styled(
            format!("{status} / {sound}"),
            Style::default().fg(accent),
        )];
        let warning = app.error.as_deref().or_else(|| {
            app.alarm
                .warning
                .as_ref()
                .map(|_| "Audio unavailable: bell fallback")
        });
        if let Some(timer) = &app.timer {
            lines.push(Line::styled(
                duration::format(timer.remaining(now)),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(if timer.status == Status::Expired {
                "Enter: dismiss alarm"
            } else {
                "Space: pause/resume"
            }));
            lines.push(Line::styled(
                warning.unwrap_or(""),
                Style::default().fg(Color::Yellow),
            ));
            lines.push(Line::from("Space pause · r restart"));
            lines.push(Line::from("m sound · q/Esc quit"));
        } else {
            lines.push(Line::styled(
                format!(
                    "› {}",
                    if app.input.is_empty() {
                        "type duration"
                    } else {
                        &app.input
                    }
                ),
                Style::default().fg(CYAN),
            ));
            lines.push(Line::from(
                ["5m", "15m", "25m"]
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        if i == app.preset {
                            format!("[{label}]")
                        } else {
                            label.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("  "),
            ));
            lines.push(Line::from("Tab preset · Enter start"));
            lines.push(Line::styled(
                warning.unwrap_or(""),
                Style::default().fg(Color::Yellow),
            ));
            lines.push(Line::from("m sound · q/Esc quit"));
        }
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
        return;
    }
    if let Some(timer) = &app.timer {
        let value = duration::format(timer.remaining(now));
        let text = if big && value.len() * 4 <= inner.width as usize {
            large_digits(&value)
        } else {
            vec![Line::from(value)]
        };
        frame.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            sections[1],
        );
        let gauge_area = centered(sections[2], inner.width.saturating_sub(8), 2);
        frame.render_widget(
            Gauge::default()
                .ratio(timer.progress(now))
                .label(format!("{} total", duration::format(timer.original)))
                .gauge_style(Style::default().fg(accent).bg(Color::DarkGray)),
            gauge_area,
        );
        let hint = match timer.status {
            Status::Running => "Timer running.",
            Status::Paused => "Timer paused. Space to resume.",
            Status::Expired => "Timer complete. Enter to dismiss.",
        };
        frame.render_widget(
            Paragraph::new(hint)
                .alignment(Alignment::Center)
                .style(Style::default().fg(DIM))
                .wrap(Wrap { trim: true }),
            sections[3],
        );
    } else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("Enter a duration or select a preset."),
                Line::from(""),
                Line::styled(
                    if app.input.is_empty() {
                        "›  type 5m, 1h30m, or seconds".into()
                    } else {
                        format!("›  {}_", app.input)
                    },
                    Style::default().fg(CYAN),
                ),
            ])
            .alignment(Alignment::Center),
            sections[1],
        );
        let presets = ["5 min", "15 min", "25 min"]
            .iter()
            .enumerate()
            .map(|(i, label)| {
                Span::styled(
                    format!("  {label}  "),
                    if i == app.preset {
                        Style::default()
                            .fg(Color::Black)
                            .bg(CYAN)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(DIM)
                    },
                )
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(Line::from(presets)).alignment(Alignment::Center),
            sections[2],
        );
        frame.render_widget(
            Paragraph::new("← / → or Tab: select preset   Enter: start")
                .alignment(Alignment::Center)
                .style(Style::default().fg(DIM))
                .wrap(Wrap { trim: true }),
            sections[3],
        );
    }
    let controls = if app.timer.is_some() {
        "Space pause/resume · r restart · m sound · q / Esc quit"
    } else {
        "Enter start · m sound · q / Esc quit"
    };
    let warning = app
        .error
        .as_deref()
        .or(app.alarm.warning.as_deref())
        .unwrap_or("");
    let footer = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(sections[4]);
    frame.render_widget(
        Paragraph::new(warning)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        footer[0],
    );
    frame.render_widget(
        Paragraph::new(controls)
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        footer[1],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use std::time::Duration;
    fn render(app: &App, width: u16, height: u16, now: Instant) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| draw(f, app, now)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    #[test]
    fn renders_entry_and_all_timer_states() {
        let now = Instant::now();
        let mut app = App::new(None);
        let entry = render(&app, 80, 24, now);
        assert!(entry.contains("Enter a duration"));
        assert!(entry.contains("15 min"));
        app.timer = Some(crate::timer::Timer::new(Duration::from_secs(60), now));
        assert!(render(&app, 80, 24, now).contains("RUNNING"));
        app.timer.as_mut().unwrap().toggle_pause(now);
        app.muted = true;
        let paused = render(&app, 80, 24, now);
        assert!(paused.contains("PAUSED"));
        assert!(paused.contains("MUTED"));
        app.timer.as_mut().unwrap().restart(now);
        app.timer
            .as_mut()
            .unwrap()
            .tick(now + Duration::from_secs(61));
        app.alarm.warning = Some("Audio unavailable: using terminal bell.".into());
        let expired = render(&app, 80, 24, now + Duration::from_secs(61));
        assert!(expired.contains("TIME'S UP"));
        assert!(expired.contains("Audio unavailable"));
    }
    #[test]
    fn resizing_and_tiny_terminals_do_not_panic() {
        let now = Instant::now();
        let app = App::new(Some(Duration::from_secs(90)));
        for (w, h) in [(1, 1), (20, 5), (28, 8), (40, 14), (80, 24), (120, 40)] {
            let result = render(&app, w, h, now);
            assert!(!result.is_empty());
        }
        assert!(render(&app, 40, 14, now).contains("01:30"));
        let compact = render(&app, 28, 8, now);
        assert!(compact.contains("01:30"));
        assert!(compact.contains("q/Esc quit"));
    }
}
