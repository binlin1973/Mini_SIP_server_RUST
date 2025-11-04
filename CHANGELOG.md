# 📜 Changelog
All notable changes to this project will be documented in this file.

This project uses **Semantic Versioning (SemVer)** and **Conventional Commit** formatting.  
Entries marked with 🤖 were **AI-assisted using OpenAI Codex (GPT-5 Codex)**.

---

## [v0.2.0] – 2025-11-05
### 🚀 Codex-Assisted Optimization 🤖

**Highlights**
- Applied full-project refactoring and optimization under guidance of **OpenAI Codex (GPT-5 Codex)**.
- Ensured clean compilation under `x86_64-unknown-linux-musl` with zero warnings.
- Verified stable SIP registration and call flow (INVITE → 180 → 200 → ACK → BYE).

**Core Updates**
- **sip_defs.rs**
  - Updated Server IP and SIP registration number ranges.  
  - Promoted magic numbers (e.g., default ports, expiry) to named constants.
- **call_map.rs**
  - Added `release_call()` to synchronize `size` counter and prevent slot leakage.  
  - Removed unsafe `unwrap()` and improved lifetime safety.
- **worker.rs**
  - Replaced teardown paths with `release_call()`.  
  - Split complex state logic and improved thread safety.
- **network_utils.rs / parsing.rs**
  - Applied Clippy recommendations (`is_some_and`, `.first()`, `derive(Default)`).
  - Replaced unwraps and magic values with safe error handling and constants.
- **main.rs**
  - Added `#![deny(warnings)]` for build-time strictness.
  - Improved startup logs and thread initialization sequence.

**Build Verification**
- `cargo check` → ✅ Passed without warnings  
- `cargo clippy --all-targets --all-features -- -D warnings` → ✅ Clean  
- `cargo build --release --target x86_64-unknown-linux-musl` → ✅ Success  
- Runtime test → ✅ Stable SIP calls established

**Acknowledgements**
> This version was collaboratively optimized with **OpenAI Codex (GPT-5 Codex)**,  
> leveraging AI-assisted static analysis, refactoring, and Rust idiom enforcement.

---

## [v0.1.0] – 2025-May-3
### Initial Release
- Implemented core SIP call-handling logic.
- Basic call state machine (INVITE / 200 OK / BYE) with multi-threaded worker design.
- Integrated minimal network handling and parsing modules.

---

📘 *Next milestone:*  
Add `clap` command-line options for server configuration (`--bind`, `--threads`, `--log-level`)  
and expand unit tests for `parsing.rs` and `network_utils.rs`.
