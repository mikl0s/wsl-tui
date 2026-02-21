# Business Requirements Document (BRD)

**Project:** WSL TUI
**Date:** 2026-02-21
**Author:** Mikkel Georgsen
**License:** MIT

---

## 1. Executive Summary

WSL TUI is an open-source terminal user interface for managing Windows Subsystem for Linux (WSL2) on Windows 11. It addresses the gap between WSL's powerful capabilities and its current CLI-only management experience. No mature TUI or GUI tool exists in the Rust ecosystem for WSL management — this project fills that void.

## 2. Problem Statement

Managing WSL2 today requires memorizing CLI commands (`wsl --list -v`, `wsl --terminate`, `wsl --export`, etc.), manually provisioning new distros with packages and configuration, and juggling multiple terminal windows. There is no unified tool that provides:

- Visual distro lifecycle management
- Automated post-install environment provisioning
- Resource monitoring across distros
- Backup/restore workflows
- Multiple connection modes (shell, embedded terminal, external terminal, Termius)

The closest tool (`wsl2-distro-manager`, Flutter) is a desktop GUI — not a TUI, not Rust, not composable.

## 3. Business Objectives

| Objective | Success Metric |
|-----------|---------------|
| Fill the WSL management gap in the Rust ecosystem | First Rust-based WSL TUI on crates.io |
| Reduce time from "WSL install" to "ready to code" | < 5 minutes from distro install to fully provisioned environment |
| Build community adoption | 500+ GitHub stars within 6 months |
| Enable extensibility | Working plugin system (Lua) with 3+ community plugins within 6 months |
| Reuse for web UI | 80%+ core logic shared between TUI and Web binaries |

## 4. Target Users

| Persona | Description | Primary Use Case |
|---------|-------------|-----------------|
| **Developer** | Manages multiple WSL distros for different projects/stacks | Quick provisioning, switching between environments |
| **Homelab Admin** | Runs services (Docker, AI, databases) in WSL | Monitoring, backup, service management |
| **Power User** | Windows user who wants WSL without memorizing CLI | Visual management, guided setup |
| **Team Lead** | Standardizes dev environments across a team | Shareable provisioning packs |

## 5. Scope

### In Scope
- Distro lifecycle management (list, install, start, stop, terminate, remove, set default)
- Stackable pack provisioning system with idempotency
- Resource monitoring (CPU, memory, disk per distro)
- Backup/restore (export/import, snapshots)
- Four connection modes: shell attach, embedded terminal, external terminal, Termius
- Dual storage backend: libsql (embedded) with JSON fallback
- Plugin system: compile-time (built-in) + runtime (Lua Phase 1, WASM Phase 2)
- Cargo workspace monorepo with shared core library
- Web UI binary (wsl-web) sharing the core library

### Out of Scope (v1)
- WSL1 management (WSL2 only)
- Remote WSL management (managing WSL on other machines)
- Pack marketplace / central registry
- Windows Store distribution
- Auto-update mechanism

## 6. Constraints

- **Platform:** Windows 11 only (WSL2 requirement)
- **Language:** Rust (for binary distribution and performance)
- **Distribution:** GitHub releases, winget, possibly scoop
- **Storage:** Must work without external database installation
- **Terminal:** Must work in Windows Terminal, ConHost, and other terminal emulators

## 7. Dependencies

- WSL2 installed and enabled on Windows 11
- `wsl.exe` available in PATH
- Terminal emulator with Unicode support (Windows Terminal recommended)
- Termius (optional, for Termius connection mode)

## 8. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| WSL CLI output format changes | Breaks parsing | Abstract WSL interaction behind trait, add integration tests |
| libsql crate compilation issues on some Windows builds | Storage unavailable | JSON fallback, tested in CI on multiple Windows versions |
| Plugin security (runtime plugins) | Malicious plugins | Permission model, sandboxing (Lua restricted env, WASM isolation) |
| Scope creep | Delayed delivery | Phased delivery: core management → provisioning → plugins |
