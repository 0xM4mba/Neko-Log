# 🌸 NekoLog // ログニャン (Anime Linux Log Viewer)

> A fast, lightweight, and fun Linux system log reader and live viewer written in **Rust**, featuring an **animated chibi anime mascot**, **sakura blossom physics**, **kawaii kaomoji status badges**, and **cyberpunk themes**!

Designed to run natively across **any Linux distribution** (Ubuntu, Debian, Fedora, Arch Linux, openSUSE, Linux Mint, Void, Alpine, etc.) on both **Wayland** and **X11**.

---

## ✨ Features

### 🐾 1. Animated Anime Mascot ("Neko-chan" / 猫ちゃん)
- **Vector-drawn procedural chibi character**: Crisp at any DPI scaling without external image files.
- **Dynamic expressions & reactions**:
  - **Nominal / Peaceful (`(✿◠‿◠)`)**: Sparkling eyes, happy smiles, contented tail swaying.
  - **Warning (`(；^ω^)`)**: Sweat-drop animations, alert perked ears.
  - **Errors / Critical Alerts (`(ノ°Д°）ノ`)**: Fiery anime eyes, panic tears, and urgent diagnostic alerts!
- **Interactive Headpats**: Click Neko-chan to pet her! Spawns floating hearts `♥` and cute anime voice lines (`"Purr purr~ Thanks for headpats, Senpai!"`).
- **Contextual Speech Bubbles**: Real-time diagnostic advice and commentary on active Linux logs.

### 🌸 2. Animated Visual Effects & Themes
- **Sakura Blossom & Sparkle Particles**: Gentle drifting sakura petals with realistic sway physics and twinkling stars. Can be toggled on/off at will.
- **Pulsating Live Stream Beacon**: Visual radar pulse showing live log ingestion velocity.
- **Multiple Anime Color Themes**:
  - `🌸 Sakura Cyberpunk` (Midnight purple, neon sakura pink, and cyan)
  - `⚡ Tokyo Neon` (Deep black, electric magenta, and neon aqua)
  - `🍵 Matcha Maid` (Forest green, matcha latte cream, and mint)
  - `🐾 Midnight Neko` (Dark violet and cozy lavender)

### 📜 3. Comprehensive Linux Log Ingestion
- **Systemd Journal (`journalctl`) Integration**:
  - `📜 All Logs`: Full system and service journals.
  - `🔥 Errors Only`: Instant filter for `-p err..emerg`.
  - `🚀 Current Boot`: Fast boot diagnostics (`-b`).
  - `🐧 Kernel Ring Buffer`: Direct access to `dmesg` kernel logs (`-k`).
- **Standard Linux Files (`/var/log/`)**:
  - Auto-discovers readable logs: `messages`, `syslog`, `boot.log`, `dnf5.log`, `pacman.log`, `auth.log`, `dpkg.log`, etc.
- **Built-in Anime File Dialog**:
  - Open any custom log file without relying on external packages like `zenity` or `kdialog`.
- **✨ Built-in NekoOS Anime Demo Stream**:
  - A built-in simulated anime mecha command center log stream for demonstrations and offline fun.

### ⚡ 4. High-Performance Log Analysis
- **Non-blocking asynchronous ingestion**: Worker threads handle log tailing and streaming via crossbeam channels; the GUI stays silky-smooth at 60 FPS.
- **Virtualized scrolling**: Effortlessly navigate through 50,000+ log lines with near-zero memory footprint.
- **Instant Filtering**:
  - Multi-level severity toggles (`🔥 ERROR`, `⚡ WARN`, `🌸 INFO`, `🐾 DEBUG`, `🍙 TRACE`).
  - Active Service / Unit selector with instant breakdown.
  - Real-time search with plain text or full regex support (`(?i)regex`).
- **Detailed Inspector Drawer**:
  - Click any log entry to view full multi-line details, copy raw output to clipboard, and view Neko-chan's automated diagnosis!
- **Exporting**: One-click export of filtered logs to formatted `JSON` or raw `TXT`.

---

## 🚀 Running NekoLog

### Instant Launch:
```bash
./nekolog
```

### Build from source:
```bash
cargo build --release
./target/release/nekolog
```

### Run tests:
```bash
cargo test
```

---

## 🐧 Why it runs on ANY Linux distro

1. **Pure Rust GUI (`eframe` + `glow`)**:
   - Compiles down to a single standalone binary.
   - Links only against standard system `libc`, `libm`, and `libgcc_s`.
   - Uses `glow` (pure OpenGL ES 2.0 / OpenGL 3.0+), which functions seamlessly on Intel, AMD, NVIDIA, and virtual machines without requiring heavy GNOME/KDE framework dependencies.
2. **Wayland & X11 Native**: Automatically selects the active display protocol at runtime.
3. **Graceful Fallbacks**: If `journalctl` isn't accessible, Neko-chan guides the user to `/var/log/` files, custom files, or the built-in demo stream.

---

## 📁 Project Architecture

```
loganalyst/
├── Cargo.toml          # Rust dependencies (eframe, egui, regex, chrono, serde)
├── nekolog             # Executable symlink to release binary
├── nekolog.desktop     # Linux XDG desktop entry
├── install.sh          # Quick install script
├── src/
│   ├── main.rs         # Application entry point, window setup & 32x32 paw icon
│   ├── app.rs          # Main UI layout, log table, filters, inspector drawer
│   ├── mascot.rs       # Animated anime chibi mascot (ears, eyes, blinking, petting)
│   ├── particles.rs    # Sakura blossom and sparkle particle physics engine
│   ├── theme.rs        # Anime color palettes and styles (Sakura, Tokyo Neon, etc.)
│   ├── log_entry.rs    # Multi-format log parser (Syslog, ISO, dmesg, brackets)
│   ├── log_source.rs   # Background journalctl runner, file tailer, demo generator
│   └── file_dialog.rs  # In-app portable file picker dialog
```
