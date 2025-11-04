# Repository Guidelines

## Project Structure & Module Organization
The crate entry point lives in `src/main.rs`, which builds the UDP SIP loop and dispatches work across threads. Shared data structures reside in `src/call_map.rs` and SIP protocol types/constants are in `src/sip_defs.rs`. Message parsing helpers sit in `src/parsing.rs`, while transport-specific routines live in `src/network_utils.rs`. Consult `State_Machine_Design.pdf` for the high-level call flow; keep documentation changes in sync with code updates. Place integration tests under `tests/` and module-level unit tests inside the corresponding `src/*.rs` file.

## Build, Test, and Development Commands
- `cargo build` compiles the server and validates dependencies.
- `cargo run` launches the UDP listener on the configured SIP port (defaults to `sip_defs::SIP_PORT`).
- `cargo fmt` enforces `rustfmt` style prior to commits.
- `cargo clippy --all-targets --all-features` surfaces common correctness and style issues.
- `cargo test` executes all unit and integration tests; use `cargo test -- --nocapture` when debugging logs.

## Coding Style & Naming Conventions
Follow Rust defaults: 4-space indentation, `snake_case` for functions and modules, `CamelCase` for types, and consts in `SCREAMING_SNAKE_CASE`. Keep modules focused—new protocol features should land in dedicated files under `src/` with clear `mod` declarations in `main.rs`. Run `cargo fmt` before pushing; address all `clippy` warnings or justify them with targeted `#[allow]` attributes.

## Testing Guidelines
Unit tests belong in `#[cfg(test)] mod tests` blocks colocated with the implementation so helpers stay private. Integration tests should exercise message flows via the public API in `tests/*.rs`; prefer descriptive function names such as `handles_cancel_request`. Add coverage for both happy-path SIP transactions and error handling (oversized packets, queue backpressure). Ensure `cargo test` passes cleanly before raising a pull request.

## Commit & Pull Request Guidelines
Commits follow a lightweight Conventional Commits style seen in history (`type: short description`). Group related changes, keep commits small, and note rationale in the body when touching networking or concurrency code. Pull requests should describe behavioral changes, reference any related issues, list manual test steps (including `cargo test`), and attach logs or packet captures when debugging network scenarios.

## Security & Configuration Tips
By default the server binds to `0.0.0.0:SIP_PORT`; adjust the listener or buffer sizes in `src/sip_defs.rs` before deploying to shared environments. Never hard-code credentials—use environment variables or configuration files ignored by Git. Validate external inputs in `parsing.rs`, and document any new trust assumptions directly in the pull request.
