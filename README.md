# Project structure

```plaintext
public-video-service/
├── Cargo.toml                  # Workspace manifest
├── crates/                     # Core services
│   ├── api-gateway/            # Control plane API (Axum)
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── origin/                 # Static file server (Axum)
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── ingest-rtmp/            # Nginx-RTMP wrapper or Rust ingest stub
│   │   ├── Dockerfile
│   │   └── nginx.conf
│   └── packager-hls/           # FFmpeg adapter (Day 1) or Rust packager (later)
│       ├── Cargo.toml
│       └── src/main.rs
│
├── deploy/docker/              # Deployment resources
│   ├── docker-compose.yml
│   ├── api-gateway.Dockerfile
│   ├── origin.Dockerfile
│   └── rtmp.Dockerfile
│
└── examples/player-hls/        # Example HLS player
    └── index.html
```
## 📦 Crates Overview

This project uses a set of core crates to build a production-ready API gateway.  
Here’s what each one does and why it’s included:

### Web stack
- **[axum](https://crates.io/crates/axum)**  
  HTTP server framework (built on Hyper). Provides routers, extractors, middleware, and response types.  
  *Why:* It’s the app’s backbone — routes, middleware, handlers.

- **[tokio](https://crates.io/crates/tokio)**  
  Async runtime for tasks, timers, and networking. Axum depends on it, and we need it for concurrent IO (proxying, DB, S3, streaming).  
  *Why:* Everything network-y is async.

- **[tower-http](https://crates.io/crates/tower-http)**  
  Ready-made HTTP middleware (CORS, compression, timeouts, tracing, auth headers, limiters, etc.).  
  *Why:* Saves weeks of re-implementing common gateway middleware.

---

### Observability & diagnostics
- **[tracing](https://crates.io/crates/tracing)**  
  Structured logging + spans. Emit machine-parsable logs and correlate work across async boundaries.  
  *Why:* Request IDs, latencies, and error fields from day one.

- **[tracing-subscriber](https://crates.io/crates/tracing-subscriber)**  
  Backend for `tracing` — formatters (JSON/plain), filters (`RUST_LOG`), layers.  
  *Why:* To actually output and filter structured logs.

---

### Error handling
- **[anyhow](https://crates.io/crates/anyhow)**  
  Ergonomic error type for application code. Great inside handlers/service logic for quick prototyping.  
  *Why:* Faster iteration, fewer custom enums while prototyping.

- **[thiserror](https://crates.io/crates/thiserror)**  
  Derive macro for clean, typed error enums and mapping them to HTTP responses.  
  *Why:* Future-proof error handling with minimal boilerplate.

---

### Serialization / payloads
- **[serde](https://crates.io/crates/serde)**  
  Derive `Serialize`/`Deserialize` for request/response structs and configs.  
  *Why:* Every API shape & config needs it.

- **[serde_json](https://crates.io/crates/serde_json)**  
  JSON (de)serialization. Works with Axum’s `Json<T>` responses.  
  *Why:* 99% of control-plane endpoints speak JSON.

---

### Configuration
- **[config](https://crates.io/crates/config)**  
  Layered config loader (env vars, YAML/TOML/JSON files, profile overrides).  
  *Why:* Switch ports, upstream URLs, and timeouts per environment without code changes.

- **[dotenvy](https://crates.io/crates/dotenvy)**  
  Load `.env` files in development for local variables like `RUST_LOG` and secrets.  
  *Why:* Smoother DX, pairs well with `config`.

---

### Outbound HTTP / proxy
- **[reqwest](https://crates.io/crates/reqwest)**  
  High-level HTTP client (TLS, redirects, gzip, timeouts).  
  *Why:* Needed for proxying to microservices and upstream services.

---

### Utilities
- **[uuid](https://crates.io/crates/uuid)**  
  Generate/parse UUIDs (v4 random or v7 time-ordered). Useful for request IDs, stream IDs, tenant IDs.  
  *Why:* Stable identifiers for logs, metrics, and cache keys.

## 🚦 Continuous Integration (CI)

- This repo uses GitHub Actions to run checks on every push and pull request:

- Format: cargo fmt --all -- --check

- Lint: cargo clippy --all-targets -- -D warnings

- Test: cargo test --all --locked
