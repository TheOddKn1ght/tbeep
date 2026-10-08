use std::time::Duration;

pub fn parse(input: &str) -> Result<Duration, String> {
    let invalid = || "Use seconds or a duration such as 5m or 1h30m10s.".to_string();
    if input.is_empty() {
        return Err(invalid());
    }
    let mut total = 0u64;
    let mut value = 0u64;
    let mut digits = false;
    let mut previous = 4;
    for c in input.chars() {
        if let Some(digit) = c.to_digit(10).filter(|_| c.is_ascii_digit()) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit as u64))
                .ok_or("Duration is too large.")?;
            digits = true;
        } else {
            let (rank, multiplier) = match c {
                'h' => (3, 3600),
                'm' => (2, 60),
                's' => (1, 1),
                _ => return Err(invalid()),
            };
            if !digits || rank >= previous {
                return Err(invalid());
            }
            total = value
                .checked_mul(multiplier)
                .and_then(|v| total.checked_add(v))
                .ok_or("Duration is too large.")?;
            previous = rank;
            value = 0;
            digits = false;
        }
    }
    if digits {
        if previous != 4 {
            return Err(invalid());
        }
        total = value;
    }
    if total == 0 {
        return Err("Please enter a positive duration.".into());
    }
    Ok(Duration::from_secs(total))
}

pub fn format(duration: Duration) -> String {
    let seconds = duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() > 0));
    let hours = seconds / 3600;
    if hours > 0 {
        format!("{hours:02}:{:02}:{:02}", seconds / 60 % 60, seconds % 60)
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_supported_durations() {
        for (input, seconds) in [
            ("10", 10),
            ("5m", 300),
            ("90s", 90),
            ("1h30m10s", 5410),
            ("1h0m", 3600),
        ] {
            assert_eq!(parse(input).unwrap().as_secs(), seconds);
        }
    }
    #[test]
    fn rejects_invalid_and_overflowing_input() {
        for input in [
            "",
            "0",
            "0s",
            "-1",
            "1.5m",
            "h",
            "1m1h",
            "1m2m",
            "1h30",
            "1s2",
            " 10",
            "1d",
            "18446744073709551616",
            "18446744073709551615h",
        ] {
            assert!(parse(input).is_err(), "accepted {input:?}");
        }
        assert!(parse("18446744073709551615").is_ok());
    }
    #[test]
    fn rounds_countdown_up() {
        assert_eq!(format(Duration::from_millis(1001)), "00:02");
        assert_eq!(format(Duration::ZERO), "00:00");
        assert_eq!(format(Duration::from_secs(3661)), "01:01:01");
    }
}
