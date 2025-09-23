# HIDra
Virtual controllers for every game.

## Layout
- `crates/` — user-mode crates (client API, FFI, IPC, tools)
- `drivers/` — KMDF/VHF drivers (added later)
- `installers/` — Suite installer, driver packages
- `docs/` — Architecture, ADRs, build notes
- `scripts/` — PowerShell helpers for build/sign/test
- `.github/workflows/` — CI

## Quick start
```powershell
cargo build