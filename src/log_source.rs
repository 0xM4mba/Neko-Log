use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::log_entry::{parse_log_line, LogEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    JournalAll,
    JournalErrors,
    JournalCurrentBoot,
    JournalKernel,
    SystemFile(PathBuf),
    CustomFile(PathBuf),
    AnimeDemo,
}

impl SourceKind {
    pub fn display_name(&self) -> String {
        match self {
            SourceKind::JournalAll => "📜 journalctl (All System Logs)".to_string(),
            SourceKind::JournalErrors => "🔥 journalctl (Errors Only: -p err..emerg)".to_string(),
            SourceKind::JournalCurrentBoot => "🚀 journalctl (Current Boot: -b)".to_string(),
            SourceKind::JournalKernel => "🐧 journalctl (Kernel Ring Buffer / dmesg)".to_string(),
            SourceKind::SystemFile(p) => format!("📄 /var/log/{}", p.file_name().unwrap_or_default().to_string_lossy()),
            SourceKind::CustomFile(p) => format!("📂 {}", p.file_name().unwrap_or_default().to_string_lossy()),
            SourceKind::AnimeDemo => "✨ NekoOS Kawaii Demo Logs".to_string(),
        }
    }
}

pub struct LogSourceManager {
    pub current_source: SourceKind,
    pub is_streaming: bool,
    pub stop_signal: Arc<AtomicBool>,
    rx: Receiver<LogEntry>,
    tx: Sender<LogEntry>,
    line_counter: usize,
}

impl LogSourceManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            current_source: SourceKind::JournalAll,
            is_streaming: false,
            stop_signal: Arc::new(AtomicBool::new(false)),
            rx,
            tx,
            line_counter: 0,
        }
    }

    pub fn discover_system_logs() -> Vec<PathBuf> {
        let mut logs = Vec::new();
        let candidates = [
            "/var/log/messages",
            "/var/log/syslog",
            "/var/log/boot.log",
            "/var/log/dnf5.log",
            "/var/log/dnf.log",
            "/var/log/pacman.log",
            "/var/log/dpkg.log",
            "/var/log/auth.log",
            "/var/log/secure",
            "/var/log/cron",
            "/var/log/dmesg",
            "/var/log/Xorg.0.log",
        ];

        for path_str in candidates {
            let p = PathBuf::from(path_str);
            if p.exists() {
                // Check if readable
                if let Ok(file) = File::open(&p) {
                    let mut reader = BufReader::new(file);
                    let mut first_line = String::new();
                    if reader.read_line(&mut first_line).is_ok() {
                        logs.push(p);
                    }
                }
            }
        }
        logs
    }

    pub fn switch_source(&mut self, source: SourceKind, stream_live: bool) {
        // Stop any running stream
        self.stop_signal.store(true, Ordering::SeqCst);

        // Replace channel
        let (tx, rx) = channel();
        self.tx = tx;
        self.rx = rx;
        self.current_source = source;
        self.line_counter = 0;
        self.is_streaming = stream_live;
        self.stop_signal = Arc::new(AtomicBool::new(false));

        let tx_clone = self.tx.clone();
        let stop_clone = self.stop_signal.clone();
        let source_clone = self.current_source.clone();

        thread::spawn(move || {
            match source_clone {
                SourceKind::JournalAll => {
                    run_journalctl(&["-n", "1500", "--no-pager"], stream_live, tx_clone, stop_clone);
                }
                SourceKind::JournalErrors => {
                    run_journalctl(&["-p", "err..emerg", "-n", "1000", "--no-pager"], stream_live, tx_clone, stop_clone);
                }
                SourceKind::JournalCurrentBoot => {
                    run_journalctl(&["-b", "-n", "1500", "--no-pager"], stream_live, tx_clone, stop_clone);
                }
                SourceKind::JournalKernel => {
                    run_journalctl(&["-k", "-n", "1500", "--no-pager"], stream_live, tx_clone, stop_clone);
                }
                SourceKind::SystemFile(path) | SourceKind::CustomFile(path) => {
                    read_file_source(&path, stream_live, tx_clone, stop_clone);
                }
                SourceKind::AnimeDemo => {
                    run_anime_demo(stream_live, tx_clone, stop_clone);
                }
            }
        });
    }

    pub fn poll_new_entries(&mut self) -> Vec<LogEntry> {
        let mut new_entries = Vec::new();
        while let Ok(mut entry) = self.rx.try_recv() {
            self.line_counter += 1;
            entry.line_number = self.line_counter;
            new_entries.push(entry);
        }
        new_entries
    }
}

fn run_journalctl(
    args: &[&str],
    stream_live: bool,
    tx: Sender<LogEntry>,
    stop: Arc<AtomicBool>,
) {
    let mut cmd = Command::new("journalctl");
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    if let Ok(mut child) = cmd.spawn() {
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut line_num = 1;
            for line_res in reader.lines() {
                if stop.load(Ordering::SeqCst) {
                    let _ = child.kill();
                    return;
                }
                if let Ok(line) = line_res {
                    if !line.trim().is_empty() {
                        let entry = parse_log_line(line_num, &line);
                        line_num += 1;
                        if tx.send(entry).is_err() {
                            return;
                        }
                    }
                }
            }
        }
        let _ = child.wait();
    } else {
        // Fallback if journalctl failed or not found
        let fallback_msg = "Notice: journalctl is not available or restricted on this host. Try opening /var/log/ files or anime demo logs!";
        let _ = tx.send(parse_log_line(1, fallback_msg));
        return;
    }

    if stream_live {
        // Follow with journalctl -f
        let mut follow_cmd = Command::new("journalctl");
        follow_cmd.args(&["-f", "-n", "0", "--no-pager"]);
        follow_cmd.stdout(Stdio::piped());
        follow_cmd.stderr(Stdio::null());

        if let Ok(mut child) = follow_cmd.spawn() {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut line_num = 1501;
                for line_res in reader.lines() {
                    if stop.load(Ordering::SeqCst) {
                        let _ = child.kill();
                        return;
                    }
                    if let Ok(line) = line_res {
                        if !line.trim().is_empty() {
                            let entry = parse_log_line(line_num, &line);
                            line_num += 1;
                            if tx.send(entry).is_err() {
                                return;
                            }
                        }
                    }
                }
            }
            let _ = child.wait();
        }
    }
}

fn read_file_source(
    path: &Path,
    stream_live: bool,
    tx: Sender<LogEntry>,
    stop: Arc<AtomicBool>,
) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            let err_line = format!(
                "Error opening log file {:?}: {} (tip: you may need read permission or sudo nya!)",
                path, e
            );
            let _ = tx.send(parse_log_line(1, &err_line));
            return;
        }
    };

    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut line_num = 1;

    // Read initial contents
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let entry = parse_log_line(line_num, &line);
        line_num += 1;
        if tx.send(entry).is_err() {
            return;
        }
        line.clear();
    }

    if !stream_live {
        return;
    }

    // Stream live file additions
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }

        line.clear();
        let bytes_read = reader.read_line(&mut line).unwrap_or(0);
        if bytes_read > 0 {
            let entry = parse_log_line(line_num, &line);
            line_num += 1;
            if tx.send(entry).is_err() {
                return;
            }
        } else {
            thread::sleep(Duration::from_millis(400));
        }
    }
}

fn run_anime_demo(
    stream_live: bool,
    tx: Sender<LogEntry>,
    stop: Arc<AtomicBool>,
) {
    let initial_samples = [
        "2026-11-24T22:00:01+09:00 neo-tokyo neko-kernel[1]: [🌸 INFO] NekoOS 6.12-anime booted successfully on Quantum Neural Core",
        "2026-11-24T22:00:02+09:00 neo-tokyo systemd[1]: Reached target Graphical Kawaii Desktop Environment.",
        "2026-11-24T22:00:03+09:00 neo-tokyo sakura-daemon[1337]: [✨ NOTE] Sakura blossom particle engine initialized at 60 FPS.",
        "2026-11-24T22:00:05+09:00 neo-tokyo miku-voice[2420]: [🌸 INFO] Vocaloid audio synthesizer connected to ALSA sound pipeline.",
        "2026-11-24T22:00:08+09:00 neo-tokyo eva-unit01[9999]: [⚡ WARN] Synchro rate dipping slightly to 98.4%! Adjusting LCL cooling valve.",
        "2026-11-24T22:00:12+09:00 neo-tokyo senpai-guard[404]: [🌸 INFO] Heartbeat check passed: Senpai is looking at the screen!",
        "2026-11-24T22:00:15+09:00 neo-tokyo matcha-cache[512]: [🌸 INFO] Matcha green tea latte buffer hit ratio: 99.8% (Cache size: 4096MB)",
        "2026-11-24T22:00:18+09:00 neo-tokyo network-neko[720]: [🐾 DEBUG] Scanning Wi-Fi SSIDs: Found 'Kawaii-Mesh-5G', 'Akihabara-Free-WiFi'",
        "2026-11-24T22:00:22+09:00 neo-tokyo ram-manager[101]: [⚡ WARN] Memory usage reached 78%! Google Chrome opened 52 anime tabs.",
        "2026-11-24T22:00:26+09:00 neo-tokyo toaster-ai[888]: [🔥 ERROR] Failed to toast mochi: Temperature sensor over 350°C!",
        "2026-11-24T22:00:30+09:00 neo-tokyo anime-guard[12]: [✨ NOTE] Auto-recovery activated: Cooled toaster down with cyber ice.",
        "2026-11-24T22:00:35+09:00 neo-tokyo neko-power[1]: [🌸 INFO] Battery state: 100% (Powered by infinite kawaii energy)",
    ];

    let mut line_num = 1;
    for s in initial_samples {
        let entry = parse_log_line(line_num, s);
        line_num += 1;
        if tx.send(entry).is_err() {
            return;
        }
    }

    if !stream_live {
        return;
    }

    let dynamic_events = [
        ("🌸 INFO", "neko-streamer[3301]", "Streaming live anime VTuber concert stream (1080p60)"),
        ("🐾 DEBUG", "catnip-scheduler[42]", "Periodic catnap scheduled in 15 minutes nya~"),
        ("✨ NOTE", "docker-sakura[190]", "Container 'waifu-generator-v2' pulled successfully"),
        ("⚡ WARN", "bento-dispatcher[77]", "Bento box delivery delayed due to intense rain in Shibuya"),
        ("🔥 ERROR", "ssh-guard[22]", "Failed password for invalid user 'baka' from 192.168.1.104 port 44321"),
        ("🌸 INFO", "kernel[0]", "CPU temperature: 41.2°C (Whisper quiet fan curve active)"),
        ("✨ NOTE", "systemd[1]", "Started Neko-chan Autonomous System Monitoring Service."),
    ];

    let mut counter = 0;
    while !stop.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(1500));
        let (lvl, svc, msg) = dynamic_events[counter % dynamic_events.len()];
        counter += 1;
        let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
        let formatted = format!("{} neo-tokyo {}: [{}] {}", ts, svc, lvl, msg);
        let entry = parse_log_line(line_num, &formatted);
        line_num += 1;
        if tx.send(entry).is_err() {
            return;
        }
    }
}
