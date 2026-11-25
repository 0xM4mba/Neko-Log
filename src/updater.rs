use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub const GITHUB_REPO: &str = "0xM4mba/Neko-Log";

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub tag_name: String,
    pub version: String,
    pub name: String,
    pub release_notes: String,
    pub download_url: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    UpdateAvailable(UpdateInfo),
    Downloading { progress: f32, message: String },
    ReadyToRestart { path: PathBuf, version: String },
    Error(String),
}

pub struct UpdateManager {
    pub current_version: String,
    pub status: UpdateStatus,
    #[allow(dead_code)]
    pub auto_check: bool,
    pub show_update_dialog: bool,
    check_rx: Option<Receiver<Result<Option<UpdateInfo>, String>>>,
    download_rx: Option<Receiver<Result<PathBuf, String>>>,
    progress_rx: Option<Receiver<f32>>,
}

impl UpdateManager {
    pub fn new() -> Self {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        Self {
            current_version,
            status: UpdateStatus::Idle,
            auto_check: true,
            show_update_dialog: false,
            check_rx: None,
            download_rx: None,
            progress_rx: None,
        }
    }

    pub fn start_check(&mut self) {
        if self.status == UpdateStatus::Checking {
            return;
        }

        self.status = UpdateStatus::Checking;
        let (tx, rx) = channel();
        self.check_rx = Some(rx);
        let curr_ver = self.current_version.clone();

        thread::spawn(move || {
            let res = check_github_release(&curr_ver);
            let _ = tx.send(res);
        });
    }

    pub fn start_download(&mut self, info: &UpdateInfo) {
        if matches!(self.status, UpdateStatus::Downloading { .. }) {
            return;
        }

        let download_url = match &info.download_url {
            Some(url) => url.clone(),
            None => {
                self.status = UpdateStatus::Error("No AppImage asset found in release.".into());
                return;
            }
        };

        self.status = UpdateStatus::Downloading {
            progress: 0.1,
            message: "Connecting to GitHub Releases...".into(),
        };

        let (progress_tx, progress_rx) = channel();
        let (res_tx, res_rx) = channel();
        self.progress_rx = Some(progress_rx);
        self.download_rx = Some(res_rx);

        thread::spawn(move || {
            let res = download_and_install(&download_url, progress_tx);
            let _ = res_tx.send(res);
        });
    }

    pub fn update(&mut self) {
        // Poll check results
        if let Some(ref rx) = self.check_rx {
            if let Ok(res) = rx.try_recv() {
                self.check_rx = None;
                match res {
                    Ok(Some(info)) => {
                        self.status = UpdateStatus::UpdateAvailable(info);
                        self.show_update_dialog = true;
                    }
                    Ok(None) => {
                        self.status = UpdateStatus::UpToDate;
                    }
                    Err(e) => {
                        self.status = UpdateStatus::Error(e);
                    }
                }
            }
        }

        // Poll progress
        if let Some(ref rx) = self.progress_rx {
            while let Ok(prog) = rx.try_recv() {
                self.status = UpdateStatus::Downloading {
                    progress: prog,
                    message: format!("Downloading update... {:.0}%", prog * 100.0),
                };
            }
        }

        // Poll download results
        if let Some(ref rx) = self.download_rx {
            if let Ok(res) = rx.try_recv() {
                self.download_rx = None;
                self.progress_rx = None;
                match res {
                    Ok(path) => {
                        let version = if let UpdateStatus::UpdateAvailable(ref info) = self.status {
                            info.version.clone()
                        } else {
                            "new".to_string()
                        };
                        self.status = UpdateStatus::ReadyToRestart { path, version };
                        self.show_update_dialog = true;
                    }
                    Err(e) => {
                        self.status = UpdateStatus::Error(e);
                    }
                }
            }
        }
    }
}

fn check_github_release(current_ver: &str) -> Result<Option<UpdateInfo>, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let output = Command::new("curl")
        .args(&[
            "-sSL",
            "-H", "User-Agent: NekoLog-App",
            "-H", "Accept: application/vnd.github.v3+json",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        return Err("Curl request failed".to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let v: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Invalid response from GitHub API: {}", e))?;

    if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
        if msg.contains("Not Found") {
            return Ok(None); // No releases published yet
        }
        if msg.contains("rate limit") {
            return Err("GitHub API rate limit reached. Try again later!".to_string());
        }
    }

    let tag_name = v.get("tag_name").and_then(|t| t.as_str()).unwrap_or("").to_string();
    if tag_name.is_empty() {
        return Ok(None);
    }

    let remote_ver = tag_name.trim_start_matches('v').to_string();
    if is_newer_version(&remote_ver, current_ver) {
        let name = v.get("name").and_then(|n| n.as_str()).unwrap_or(&tag_name).to_string();
        let release_notes = v.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string();
        let html_url = v.get("html_url").and_then(|h| h.as_str()).unwrap_or("").to_string();

        let mut download_url = None;
        if let Some(assets) = v.get("assets").and_then(|a| a.as_array()) {
            for asset in assets {
                if let Some(asset_name) = asset.get("name").and_then(|n| n.as_str()) {
                    if asset_name.ends_with(".AppImage") || asset_name.ends_with(".appimage") {
                        if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                            download_url = Some(url.to_string());
                            break;
                        }
                    }
                }
            }
        }

        Ok(Some(UpdateInfo {
            tag_name,
            version: remote_ver,
            name,
            release_notes,
            download_url,
            html_url,
        }))
    } else {
        Ok(None)
    }
}

pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_ver = |s: &str| -> (u32, u32, u32) {
        let clean = s.trim_start_matches('v');
        let parts: Vec<&str> = clean.split('.').collect();
        let major = parts.get(0).and_then(|x| x.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|x| x.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let r = parse_ver(remote);
    let c = parse_ver(current);

    r > c
}

fn download_and_install(download_url: &str, progress_tx: Sender<f32>) -> Result<PathBuf, String> {
    // 1. Identify target file path
    // If running as an AppImage, std::env::var("APPIMAGE") contains the path to the current .AppImage!
    let target_path = if let Ok(appimage_env) = std::env::var("APPIMAGE") {
        PathBuf::from(appimage_env)
    } else {
        std::env::current_exe().map_err(|e| format!("Cannot locate current executable: {}", e))?
    };

    let temp_target = target_path.with_extension("update.tmp");

    let _ = progress_tx.send(0.25);

    // 2. Download via curl
    let status = Command::new("curl")
        .args(&[
            "-L",
            "-o",
            temp_target.to_string_lossy().as_ref(),
            download_url,
        ])
        .status()
        .map_err(|e| format!("Curl execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_target);
        return Err("Download failed. Check your internet connection.".to_string());
    }

    let _ = progress_tx.send(0.85);

    // 3. Make downloaded file executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&temp_target) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&temp_target, perms);
        }
    }

    // 4. Atomically replace the existing AppImage/binary
    fs::rename(&temp_target, &target_path)
        .map_err(|e| format!("Failed to apply update (permission error?): {}", e))?;

    let _ = progress_tx.send(1.0);

    Ok(target_path)
}

pub fn restart_application(path: &Path) {
    let _ = Command::new(path).spawn();
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.1", "1.0.0"));
        assert!(is_newer_version("v1.1.0", "1.0.9"));
        assert!(is_newer_version("2.0.0", "1.99.99"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.1"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
    }
}
