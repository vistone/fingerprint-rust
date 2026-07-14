# AGENTS.md

## Cursor Cloud specific instructions

This repo is primarily a **Rust Cargo workspace** (`fingerprint-rust`) for browser/TLS/HTTP
fingerprint generation and analysis. There is **no frontend/web UI**; development is
library/CLI/service oriented. A separate optional Python FastAPI service lives in `phase7_api/`.

### Toolchain & system dependencies
- Rust is pinned to **1.90.0** via `rust-toolchain.toml` (rustup auto-selects it; `rustfmt` + `clippy` components are declared there).
- Building requires these **system packages** (already installed in the VM snapshot): `pkg-config`, `libssl-dev` (the `openssl-sys` crate fails to build without them), and `redis-server` (only needed to run the gateway). These are OS-level and are NOT part of the update script; if a fresh VM lacks them, install with `sudo apt-get install -y pkg-config libssl-dev redis-server`.

### Build / lint / test (standard commands; see `README.md` and `CONTRIBUTING.md`)
- `cargo build` / `cargo check` at the repo root only target the **stable default members** (see `default-members` in `Cargo.toml`). Use `--workspace` to include preview/prototype crates.
- Lint: `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Tests: `cargo test --workspace --lib` (all pass). Some network-facing tests are `#[ignore]` and require internet/DNS.
- Full workspace build/test is heavier than the default set because it compiles all 22 crates.

### Running examples (non-obvious)
- The root `examples/` directory is **not** a compiled Cargo target (the workspace root has no `[package]`). Do not expect `cargo run --example basic` from root to work.
- Examples belong to individual crates. Run them with `-p`, e.g.
  `cargo run -p fingerprint-core --example modern_fingerprinting` (a good end-to-end smoke test of core fingerprint generation: JA4/JA4X/PQC/WASM).
- Examples under `crates/fingerprint/examples/` are `*.rs.disabled` and won't build as-is.

### Gateway service (`crates/fingerprint-gateway`, optional)
- Requires Redis. Start it with `redis-server --daemonize yes --save ""`, then run `cargo run -p fingerprint-gateway --bin gateway`.
- Env vars: `GATEWAY_HOST` (default `0.0.0.0`), `GATEWAY_PORT` (default `8080`), `REDIS_URL` (default `redis://127.0.0.1:6379`). Default test API key is `sk_test_demo123`.
- Known pre-existing bug: `run_server` registers `web::Data::new(Arc<RateLimiter>)` / `Arc<ApiKeyValidator>` while handlers extract `web::Data<RateLimiter>` / `web::Data<ApiKeyValidator>`. This type mismatch makes every route return HTTP 500 ("Requested application data is not configured correctly"). The service still builds, boots, binds :8080, and connects to Redis. Fixing it is an application-code change, not environment setup.

### Phase 7 Python API (`phase7_api/`, optional)
- FastAPI service run with `python3 -m uvicorn app.main:app --reload --port 8000` (see `phase7_api/Makefile`). Real ML inference needs trained `.pkl` model files that are **not committed** to the repo (only `models/feature_info.json` exists), so full identify/validate endpoints cannot run without supplying models.
