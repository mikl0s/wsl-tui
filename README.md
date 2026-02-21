<div align="center">

```
                 ██╗    ██╗███████╗██╗         ████████╗██╗   ██╗██╗
                 ██║    ██║██╔════╝██║         ╚══██╔═╝██║   ██║██║
                 ██║ █╗ ██║███████╗██║            ██║   ██║   ██║██║
                 ██║███╗██║╚════██║██║            ██║   ██║   ██║██║
                 ╚███╔███╔╝███████║███████╗       ██║   ╚██████╔╝██║
                  ╚══╝╚══╝ ╚══════╝╚══════╝       ╚═╝    ╚═════╝ ╚═╝
```

### The WSL2 manager your terminal deserves.

[![MIT License](https://img.shields.io/badge/license-MIT-cba6f7?style=for-the-badge&logo=opensourceinitiative&logoColor=white)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-fab387?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Windows 11](https://img.shields.io/badge/windows%2011-89b4fa?style=for-the-badge&logo=windows11&logoColor=white)](https://www.microsoft.com/windows)
[![Ratatui](https://img.shields.io/badge/ratatui-0.30+-a6e3a1?style=for-the-badge)](https://ratatui.rs/)
[![Stars](https://img.shields.io/github/stars/mikkelens/wsl-tui?style=for-the-badge&color=f9e2af&logo=github)](https://github.com/mikkelens/wsl-tui/stargazers)

**Manage. Provision. Monitor. Connect.**
<br>
All your WSL2 distros. One beautiful TUI.

[Get Started](#-quickstart) · [Features](#-features) · [Packs](#-stackable-packs) · [Plugins](#-plugins) · [Web UI](#-web-ui) · [Contributing](#-contributing)

---

</div>

<br>

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  WSL TUI v0.1.0                                 ▸ Ubuntu (Running)    ◆ 14:32  │
├──────────────────────┬──────────────────────────────────────────────────────────┤
│                      │                                                          │
│  DISTROS             │  DETAILS                                                 │
│  ───────             │  ────────                                                │
│  ▸ Ubuntu 24.04  ●   │  Name:     Ubuntu-24.04                                  │
│    Debian 12     ●   │  State:    Running                                       │
│    Fedora 41     ○   │  WSL:      2                                             │
│    Alpine 3.19   ○   │  User:     mikkel                                        │
│                      │  IP:       172.28.0.2                                    │
│                      │  Uptime:   2h 14m                                        │
│                      │  Memory:   1.2 GB / 4 GB                                 │
│                      │  Disk:     8.4 GB / 256 GB                               │
│  QUICK ACTIONS       │                                                          │
│  ────────────        │  ┌─ RESOURCE MONITOR ────────────────────────────────┐   │
│  [s] Start           │  │  CPU  ▓▓▓▓▓▓░░░░░░░░░░  32%                     │   │
│  [x] Stop            │  │  MEM  ▓▓▓▓▓▓▓▓░░░░░░░░  48%                     │   │
│  [c] Connect         │  │  DISK ▓▓▓░░░░░░░░░░░░░░  18%                     │   │
│  [d] Set Default     │  └──────────────────────────────────────────────────┘   │
│  [p] Provision       │                                                          │
│  [b] Backup          │                                                          │
│  [i] Install New     │                                                          │
│                      │                                                          │
├──────────────────────┴──────────────────────────────────────────────────────────┤
│  [Tab] Panel   [/] Search   [?] Help   [:] Command   [q] Quit      ◉ libsql   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

<br>

## Why WSL TUI?

You installed WSL. You ran `wsl --install Ubuntu`. Now what?

You Google "set up dev environment WSL." You copy-paste 47 commands. You forget to install nvm. You mess up your .zshrc. You do it again next week for Debian. And again when you nuke your distro.

**WSL TUI fixes this.**

| Without WSL TUI | With WSL TUI |
|:---|:---|
| Memorize `wsl --list -v`, `--terminate`, `--export`... | Visual dashboard with one-key actions |
| Copy-paste setup scripts from 12 blog posts | Select packs → apply → done |
| "Wait, is my distro running?" | Real-time status + resource monitoring |
| 5 terminal windows open | Embedded terminal, Termius, or your preferred app |
| "I should have backed up before that..." | One-key export/snapshot |
| Every distro is a unique snowflake | Reproducible, idempotent provisioning |

<br>

## ✨ Features

### Distro Management
Full lifecycle control — install, start, stop, terminate, remove, set default. Everything `wsl.exe` can do, but you don't have to remember how.

### 📦 Stackable Packs
The killer feature. Post-install provisioning via composable, idempotent TOML packs. Pick your distro, stack your packs, hit go. **Ready to code in minutes, not hours.**

### 📊 Resource Monitoring
Real-time CPU, memory, and disk usage per distro. Gauges, charts, and historical data. Know what's eating your RAM.

### 🔌 4 Connection Modes
Connect your way:
- **Shell Attach** — Drop into a shell, TUI waits
- **Embedded Terminal** — Split pane, stay in the TUI
- **External Terminal** — Launch in Windows Terminal, WezTerm, Alacritty
- **Termius** — Full SSH experience with auto-provisioned `sshd`

### 💾 Backup & Restore
Export to `.tar` or `.vhdx`. Import. Snapshot. Never lose a configured distro again.

### 🧩 Plugin System
Built-in plugins for core features. Extend with Lua scripts (Phase 1) or WASM modules (Phase 2). Community-extensible with sandboxed execution.

### 🗄️ Smart Storage
Embedded libsql database — no install needed. Auto-falls back to JSON if needed. Override in config. Inspect with the libsql CLI if you're that kind of person.

<br>

## 📦 Stackable Packs

This is what makes WSL TUI different. Packs are composable layers of configuration — like Docker layers, but for your WSL environment.

```
┌──────────────── PROVISION: Ubuntu 24.04 ─────────────────────────────┐
│                                                                       │
│  Select packs to apply:                                              │
│                                                                       │
│    ☑ home-setup       Shell, prompt, editor, SSH keys, dotfiles      │
│    ☑ claude-code      Claude Code CLI + configuration                │
│    ☑ nvm-node         nvm + Node LTS + global packages               │
│    ☐ python-dev       pyenv + Python + poetry/uv                     │
│    ☐ rust-dev         rustup + stable toolchain + cargo tools        │
│    ☐ docker           Docker Engine + compose                        │
│    ☐ ai-stack         ollama + open-webui + GPU passthrough          │
│    ☐ gui-desktop      X11/Wayland forwarding + GUI apps              │
│    ☐ server-base      nginx/caddy + certbot + firewall               │
│                                                                       │
│  [Space] Toggle   [Enter] Apply   [p] Preview   [Esc] Cancel        │
└───────────────────────────────────────────────────────────────────────┘
```

### How It Works

1. **Select packs** from the provisioning screen (multi-select)
2. **Answer variables** — each pack prompts for your preferences (which shell? which Node version?)
3. **Dependencies resolve** automatically (nvm-node auto-includes base packages)
4. **Idempotent execution** — re-run anytime. Already installed? Skipped. Config drifted? Corrected.
5. **Dry-run mode** — preview exactly what will change before applying

### Built-in Packs

| Pack | What It Sets Up |
|:-----|:----------------|
| `home-setup` | Zsh/Fish/Bash + Starship/P10k + Neovim/Helix + SSH key sync + dotfiles |
| `claude-code` | Claude Code CLI, ready to use |
| `nvm-node` | nvm + Node.js LTS + your choice of global packages |
| `python-dev` | pyenv + Python + poetry or uv |
| `rust-dev` | rustup + stable + cargo-watch, cargo-edit, etc. |
| `docker` | Docker Engine + Compose, daemon configured |
| `ai-stack` | Ollama + Open WebUI + GPU passthrough setup |
| `gui-desktop` | X11/Wayland forwarding for GUI apps |
| `server-base` | Nginx or Caddy + Let's Encrypt + UFW |

### Create Your Own

Packs are just TOML files. Drop them in `~/.wsl-tui/packs/`:

```toml
[pack]
id = "my-tools"
name = "My Dev Tools"
description = "My personal toolkit"
category = "dev"
depends_on = ["home-setup"]

[[steps]]
type = "package_install"
packages = ["ripgrep", "fd-find", "bat", "eza", "zoxide", "fzf"]

[[steps]]
type = "script_run"
script = "curl -sS https://starship.rs/install.sh | sh -s -- -y"

[[steps]]
type = "file_write"
path = "~/.config/starship.toml"
content = """
[character]
success_symbol = "[➜](bold green)"
"""
```

<br>

## 🚀 Quickstart

### Install

```bash
# Via winget (recommended)
winget install wsl-tui

# Via scoop
scoop install wsl-tui

# Via cargo
cargo install wsl-tui

# Or download the binary from GitHub Releases
```

### Run

```bash
wsl-tui
```

That's it. No config needed. It auto-detects your distros, sets up storage, and drops you into the dashboard.

### First Time?

1. Launch `wsl-tui`
2. Press `i` to install a new distro
3. Select your distro (Ubuntu 24.04, Debian 12, etc.)
4. Select packs to apply (`home-setup` + `claude-code` + `nvm-node`)
5. Answer the wizard prompts (shell preference, Node version, etc.)
6. Watch it provision in real-time
7. Press `c` to connect — you're in a fully configured environment

<br>

## 🧩 Plugins

### Built-in Plugins

Core functionality is delivered via compile-time plugins:

| Plugin | Provides |
|:-------|:---------|
| `wsl-plugin-distro` | Distro lifecycle (install, start, stop, remove, default) |
| `wsl-plugin-provision` | Pack provisioning engine + wizard UI |
| `wsl-plugin-monitor` | Resource monitoring + charts |
| `wsl-plugin-backup` | Export, import, snapshot management |
| `wsl-plugin-connect` | Shell, embedded terminal, external, Termius |

### Lua Plugins (Phase 1)

Drop `.lua` files in `~/.wsl-tui/plugins/`:

```lua
-- my-plugin.lua
return {
    name = "my-plugin",
    description = "Does something cool",
    version = "0.1.0",

    commands = {
        {
            name = "hello",
            description = "Say hello",
            execute = function(ctx)
                ctx.notify("Hello from my plugin!")
            end
        }
    }
}
```

### WASM Plugins (Phase 2)

Compile any language to `.wasm`, drop in `~/.wsl-tui/plugins/`. Sandboxed, permission-gated, cross-platform.

<br>

## 🌐 Web UI

WSL TUI ships as a **monorepo with two binaries**:

| Binary | Interface | Stack |
|:-------|:----------|:------|
| `wsl-tui` | Terminal (Ratatui) | Ratatui + crossterm |
| `wsl-web` | Browser (SPA) | Axum + embedded frontend |

Both share `wsl-core` — the same plugin system, provisioning engine, storage, and WSL interaction layer. Same features, different interfaces.

```bash
# Run the web UI
wsl-web

# Opens http://127.0.0.1:3000
```

<br>

## 🏗️ Architecture

```
wsl-tui/
├── crates/
│   ├── wsl-core/                 # Shared library (plugins, WSL API, storage, packs)
│   ├── wsl-plugin-distro/        # Distro management
│   ├── wsl-plugin-provision/     # Pack provisioning
│   ├── wsl-plugin-monitor/       # Resource monitoring
│   ├── wsl-plugin-backup/        # Backup/restore
│   ├── wsl-plugin-connect/       # Connection modes
│   ├── wsl-tui/                  # Terminal UI binary
│   └── wsl-web/                  # Web UI binary
├── packs/                        # Built-in provisioning packs
└── docs/                         # Design docs, PRD, SOW
```

### Tech Stack

| Component | Technology |
|:----------|:-----------|
| Language | Rust (stable) |
| TUI | Ratatui 0.30+ / crossterm |
| Web | Axum 0.8+ / tower-http |
| Async | Tokio |
| Database | libsql (embedded) with JSON fallback |
| Config | TOML |
| CLI | clap 4.x |
| Plugins | mlua (Lua) → wasmtime (WASM) |
| Theme | Catppuccin Mocha |

<br>

## ⌨️ Keybindings

| Key | Action |
|:----|:-------|
| `j` / `k` | Navigate up/down |
| `h` / `l` | Navigate left/right |
| `Tab` | Switch panel |
| `Enter` | Select / confirm |
| `s` | Start distro |
| `x` | Stop distro |
| `c` | Connect to distro |
| `d` | Set as default |
| `p` | Provision (open pack selector) |
| `b` | Backup / export |
| `i` | Install new distro |
| `/` | Search / filter |
| `:` | Command palette |
| `?` | Help |
| `1`-`5` | Jump to view (Dashboard, Provision, Monitor, Backup, Logs) |
| `q` | Quit |

<br>

## 🎨 Theme

Built on **Catppuccin Mocha** — the internet's favorite dark palette.

```
  Mauve     Blue      Green     Yellow    Red       Peach     Teal      Pink
  #cba6f7   #89b4fa   #a6e3a1   #f9e2af   #f38ba8   #fab387   #94e2d5   #f5c2e7
  ███████   ███████   ███████   ███████   ███████   ███████   ███████   ███████
  Focus     Links     Running   Warning   Error     Search    Connect   Provision

  Base      Surface0  Surface1  Text      Subtext1  Lavender  Sapphire  Flamingo
  #1e1e2e   #313244   #45475a   #cdd6f4   #bac2de   #b4befe   #74c7ec   #f2cdcd
  ███████   ███████   ███████   ███████   ███████   ███████   ███████   ███████
  BG        Panels    Borders   Primary   Secondary Headings  Info      CPU
```

<br>

## 📁 Configuration

All config lives in `~/.wsl-tui/`:

```
~/.wsl-tui/
├── config.toml          # Main configuration
├── data.db              # libsql database
├── packs/               # Custom packs
├── plugins/             # Lua/WASM plugins
├── logs/                # Execution logs
└── backups/             # Distro snapshots
```

### Example `config.toml`

```toml
[general]
storage_backend = "auto"    # "auto" | "libsql" | "json"
refresh_interval = 5        # seconds

[connection]
default_mode = "shell_attach"
external_terminal = "wt.exe"
external_args = "-p {distro_name}"

[connection.termius]
auto_setup_ssh = true
base_port = 2222

[theme]
palette = "catppuccin-mocha"

[keybindings]
quit = "q"
search = "/"
help = "?"
```

<br>

## 🤝 Contributing

Contributions are welcome! This project is MIT licensed and open to all.

### Development Setup

```bash
# Clone
git clone https://github.com/mikkelens/wsl-tui.git
cd wsl-tui

# Build
cargo build --workspace

# Run
cargo run -p wsl-tui

# Test
cargo test --workspace

# Lint
cargo clippy --workspace
```

### Areas for Contribution

- **Built-in packs** — Add more provisioning packs for common setups
- **Lua plugins** — Build community plugins
- **Themes** — Port other Catppuccin flavors (Latte, Frappe, Macchiato)
- **Documentation** — Guides, tutorials, pack authoring docs
- **Testing** — Integration tests, UI tests, cross-terminal testing

<br>

## 📄 License

[MIT](LICENSE) — Mikkel Georgsen, 2026

<div align="center">

---

**Built with Rust.** Themed with Catppuccin. Made for Windows 11.

If this tool saves you time, consider giving it a ⭐

</div>
