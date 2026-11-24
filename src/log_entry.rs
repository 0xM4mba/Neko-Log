use egui::Color32;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::OnceLock;
use crate::theme::AnimePalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Critical,
    Fatal,
    Unknown,
}

impl LogLevel {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Notice => "NOTICE",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
            LogLevel::Fatal => "FATAL",
            LogLevel::Unknown => "INFO",
        }
    }

    pub fn anime_badge(&self) -> &'static str {
        match self {
            LogLevel::Trace => "🍙 TRACE",
            LogLevel::Debug => "🐾 DEBUG",
            LogLevel::Info => "🌸 INFO",
            LogLevel::Notice => "✨ NOTE",
            LogLevel::Warn => "⚡ WARN",
            LogLevel::Error => "🔥 ERROR",
            LogLevel::Critical => "💥 CRIT",
            LogLevel::Fatal => "💀 FATAL",
            LogLevel::Unknown => "🌱 LOG",
        }
    }

    pub fn anime_kaomoji(&self) -> &'static str {
        match self {
            LogLevel::Trace => "(・ω・)",
            LogLevel::Debug => "( =^･ω･^= )",
            LogLevel::Info => "(✿◠‿◠)",
            LogLevel::Notice => "(*^▽^*)",
            LogLevel::Warn => "(；^ω^)",
            LogLevel::Error => "(ノ°Д°）ノ",
            LogLevel::Critical => "(((( ;°Д°))))",
            LogLevel::Fatal => "(x_x;)",
            LogLevel::Unknown => "(・∀・)",
        }
    }

    pub fn color(&self, p: &AnimePalette) -> Color32 {
        match self {
            LogLevel::Trace => p.text_muted,
            LogLevel::Debug => p.accent_purple,
            LogLevel::Info => p.accent_cyan,
            LogLevel::Notice => p.accent_green,
            LogLevel::Warn => p.accent_yellow,
            LogLevel::Error => p.accent_red,
            LogLevel::Critical => Color32::from_rgb(255, 40, 100),
            LogLevel::Fatal => Color32::from_rgb(255, 0, 80),
            LogLevel::Unknown => p.text_primary,
        }
    }

    #[allow(dead_code)]
    pub fn bg_color(&self, p: &AnimePalette) -> Color32 {
        let c = self.color(p);
        Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 35)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub line_number: usize,
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub raw: String,
}

static SYSLOG_RE: OnceLock<Regex> = OnceLock::new();
static ISO_RE: OnceLock<Regex> = OnceLock::new();
static BRACKET_RE: OnceLock<Regex> = OnceLock::new();
static DMESG_RE: OnceLock<Regex> = OnceLock::new();

pub fn parse_log_line(line_number: usize, raw: &str) -> LogEntry {
    let raw_str = raw.trim_end();
    if raw_str.is_empty() {
        return LogEntry {
            line_number,
            timestamp: String::new(),
            level: LogLevel::Unknown,
            service: "system".into(),
            message: String::new(),
            raw: raw.to_string(),
        };
    }

    // 1. Try ISO timestamp regex: "2026-11-24T22:01:21.123+00:00 hostname service[pid]: msg"
    let iso_re = ISO_RE.get_or_init(|| {
        Regex::new(r"^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:[+-]\d{2}:?\d{2}|Z)?)\s+(?:[\w.-]+\s+)?([a-zA-Z0-9_.-]+(?:\[\d+\])?):\s*(.*)$").unwrap()
    });
    if let Some(caps) = iso_re.captures(raw_str) {
        let ts = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let svc = caps.get(2).map(|m| m.as_str()).unwrap_or("system").to_string();
        let msg = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
        let level = detect_level(&msg, &svc);
        return LogEntry {
            line_number,
            timestamp: ts,
            level,
            service: svc,
            message: msg,
            raw: raw.to_string(),
        };
    }

    // 2. Try Standard Syslog: "Nov 24 22:01:21 fedora systemd[1]: msg"
    let syslog_re = SYSLOG_RE.get_or_init(|| {
        Regex::new(r"^([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+([a-zA-Z0-9_.-]+)\s+([a-zA-Z0-9_.-]+(?:\[\d+\])?):\s*(.*)$").unwrap()
    });
    if let Some(caps) = syslog_re.captures(raw_str) {
        let ts = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let svc = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
        let msg = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();
        let level = detect_level(&msg, &svc);
        return LogEntry {
            line_number,
            timestamp: ts,
            level,
            service: svc,
            message: msg,
            raw: raw.to_string(),
        };
    }

    // 3. Try dmesg format: "[   12.345678] service: message"
    let dmesg_re = DMESG_RE.get_or_init(|| {
        Regex::new(r"^\[\s*(\d+\.\d+)\]\s*(?:([a-zA-Z0-9_. -]+?):\s+)?(.*)$").unwrap()
    });
    if let Some(caps) = dmesg_re.captures(raw_str) {
        let ts = format!("+{}s", caps.get(1).map(|m| m.as_str()).unwrap_or("0"));
        let svc = caps.get(2).map(|m| m.as_str()).unwrap_or("kernel").to_string();
        let msg = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
        let level = detect_level(&msg, &svc);
        return LogEntry {
            line_number,
            timestamp: ts,
            level,
            service: svc,
            message: msg,
            raw: raw.to_string(),
        };
    }

    // 4. Try Bracketed format: "[2026-11-24 22:01:21] [INFO] [service]: message"
    let bracket_re = BRACKET_RE.get_or_init(|| {
        Regex::new(r"^\[([^\]]+)\]\s*(?:\[([A-Za-z0-9_-]+)\])?\s*(?:\[([A-Za-z0-9_-]+)\])?\s*[:\-]?\s*(.*)$").unwrap()
    });
    if let Some(caps) = bracket_re.captures(raw_str) {
        let ts = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let p2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let p3 = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let msg = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();

        let (level, svc) = if !p2.is_empty() && is_level_str(p2) {
            (parse_level_str(p2), if !p3.is_empty() { p3.to_string() } else { "app".into() })
        } else if !p3.is_empty() && is_level_str(p3) {
            (parse_level_str(p3), p2.to_string())
        } else {
            (detect_level(&msg, p2), if !p2.is_empty() { p2.to_string() } else { "app".into() })
        };

        return LogEntry {
            line_number,
            timestamp: ts,
            level,
            service: svc,
            message: msg,
            raw: raw.to_string(),
        };
    }

    // Fallback: entire line is message
    let level = detect_level(raw_str, "");
    LogEntry {
        line_number,
        timestamp: String::new(),
        level,
        service: "system".into(),
        message: raw_str.to_string(),
        raw: raw.to_string(),
    }
}

fn is_level_str(s: &str) -> bool {
    let upper = s.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "EMERG" | "EMERGENCY" | "FATAL" | "CRIT" | "CRITICAL" | "ERR" | "ERROR" |
        "WARN" | "WARNING" | "NOTICE" | "INFO" | "DEBUG" | "TRACE"
    )
}

fn parse_level_str(s: &str) -> LogLevel {
    let upper = s.to_ascii_uppercase();
    match upper.as_str() {
        "EMERG" | "EMERGENCY" | "FATAL" => LogLevel::Fatal,
        "CRIT" | "CRITICAL" => LogLevel::Critical,
        "ERR" | "ERROR" => LogLevel::Error,
        "WARN" | "WARNING" => LogLevel::Warn,
        "NOTICE" => LogLevel::Notice,
        "INFO" => LogLevel::Info,
        "DEBUG" => LogLevel::Debug,
        "TRACE" => LogLevel::Trace,
        _ => LogLevel::Info,
    }
}

pub fn detect_level(msg: &str, service: &str) -> LogLevel {
    let lower_msg = msg.to_ascii_lowercase();
    let lower_svc = service.to_ascii_lowercase();

    if lower_msg.contains("fatal") || lower_msg.contains("panic") || lower_msg.contains("oom-killer") {
        return LogLevel::Fatal;
    }
    if lower_msg.contains("critical") || lower_msg.contains("segfault") || lower_msg.contains("core dumped") {
        return LogLevel::Critical;
    }
    if lower_msg.contains("error")
        || lower_msg.contains("failed")
        || lower_msg.contains("failure")
        || lower_msg.contains("err:")
        || lower_msg.contains("[err]")
        || lower_svc.contains("error")
    {
        return LogLevel::Error;
    }
    if lower_msg.contains("warn")
        || lower_msg.contains("warning")
        || lower_msg.contains("deprecated")
        || lower_msg.contains("retry")
    {
        return LogLevel::Warn;
    }
    if lower_msg.contains("notice") || lower_msg.contains("started") || lower_msg.contains("reached target") {
        return LogLevel::Notice;
    }
    if lower_msg.contains("debug") {
        return LogLevel::Debug;
    }
    if lower_msg.contains("trace") {
        return LogLevel::Trace;
    }

    LogLevel::Info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_syslog_line() {
        let line = "Nov 24 22:01:21 fedora systemd[1]: systemd-hostnamed.service: Deactivated successfully.";
        let entry = parse_log_line(1, line);
        assert_eq!(entry.line_number, 1);
        assert_eq!(entry.timestamp, "Nov 24 22:01:21");
        assert_eq!(entry.service, "systemd[1]");
        assert!(entry.message.contains("Deactivated successfully"));
        assert_eq!(entry.level, LogLevel::Info);
    }

    #[test]
    fn test_parse_syslog_error() {
        let line = "Nov 24 22:01:19 fedora brave.desktop[5349]: Hash Manager Error : 1 failed to open file";
        let entry = parse_log_line(2, line);
        assert_eq!(entry.line_number, 2);
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.level.anime_badge(), "🔥 ERROR");
    }

    #[test]
    fn test_parse_iso_line() {
        let line = "2026-11-24T22:00:08+09:00 neo-tokyo eva-unit01[9999]: [⚡ WARN] Synchro rate dipping slightly";
        let entry = parse_log_line(3, line);
        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.level.anime_badge(), "⚡ WARN");
    }

    #[test]
    fn test_parse_dmesg_line() {
        let line = "[   12.345678] usb 1-1: USB disconnect, address 3";
        let entry = parse_log_line(4, line);
        assert_eq!(entry.timestamp, "+12.345678s");
        assert_eq!(entry.service, "usb 1-1");
    }

    #[test]
    fn test_detect_fatal_panic() {
        let line = "Kernel panic - not syncing: Fatal exception in interrupt";
        let entry = parse_log_line(5, line);
        assert_eq!(entry.level, LogLevel::Fatal);
        assert_eq!(entry.level.anime_badge(), "💀 FATAL");
    }
}
