## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (see `rust-toolchain.toml`)
- Cargo

### Development Setup
```bash
# Clone and navigate to the project
git clone <repository-url>
cd public-video-service

# Run the API Gateway
cargo run -p api-gateway

# Or use the Makefile
make run
```

### Configuration
The API Gateway supports flexible configuration through multiple sources with precedence:
**defaults < config file < environment variables**

#### Configuration File (`config.toml`)
```toml
# Server configuration
host = "127.0.0.1"              # Bind address (empty = all interfaces)
port = 3000                     # Server port
request_timeout_ms = 15000       # Request timeout in milliseconds

# CORS configuration
cors_origins = [
    "http://localhost:3000",
    "https://app.example.com"
]

# Upstream services
[upstreams]
user_service = "http://localhost:3001"
auth_service = "http://localhost:3002"
video_service = "http://localhost:3003"
```

#### Environment Variables
```bash
# Override any config file values
export APP_HOST="0.0.0.0"
export APP_PORT="8080"
export APP_REQUEST_TIMEOUT_MS="30000"
export APP_CORS_ORIGINS='["https://production.example.com"]'
export APP_UPSTREAMS__USER_SERVICE="http://user-service:3001"
```

#### Example Configuration
See `config.dev.toml` for a complete example with detailed comments.

### Running the Application
```bash
# Run with default configuration
cargo run -p api-gateway

# Run with custom config file
cargo run -p api-gateway -- --config custom.toml

# Run with environment variables
APP_PORT=8080 APP_HOST=0.0.0.0 cargo run -p api-gateway

# Run with debug logging
RUST_LOG=debug cargo run -p api-gateway
```

## Project structure

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
│   └── packager-hls/           # FFmpeg adapter 
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

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p api-gateway

# Run tests with output
cargo test -- --nocapture

# Run tests sequentially (recommended for integration tests)
cargo test -- --test-threads=1
```

### Test Categories

#### Unit Tests
- Located in `src/` files alongside source code
- Test individual functions and modules
- Run automatically with `cargo test`

#### Integration Tests
- Located in `tests/` directory
- Test complete functionality and configuration loading
- **Important**: Run with `--test-threads=1` to avoid test interference

### Configuration Testing
The API Gateway includes comprehensive configuration tests that verify:

- **Default values** are used when no config is provided
- **File configuration** overrides defaults correctly
- **Environment variables** override file configuration
- **Validation** catches invalid configuration values
- **Test isolation** prevents interference between tests

#### Running Configuration Tests
```bash
# Run all configuration tests
cargo test -p api-gateway -- --test-threads=1

# Run specific configuration test
cargo test -p api-gateway test_config_defaults_only

# Run with debug output
cargo test -p api-gateway -- --nocapture --test-threads=1
```

### Test Isolation
**Important**: Configuration tests use test-specific config files to prevent interference. Always run tests sequentially:

```bash
# ✅ Correct - Sequential execution
cargo test -- --test-threads=1

# ❌ Avoid - Parallel execution can cause test interference
cargo test
```

### Test Coverage
```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 🚦 Continuous Integration (CI)

This repo uses GitHub Actions to run checks on every push and pull request:

- **Format**: `cargo fmt --all -- --check`
- **Lint**: `cargo clippy --all-targets -- -D warnings`
- **Test**: `cargo test --all --locked -- --test-threads=1`

### Pre-commit Checks
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets

# Run tests
cargo test -- --test-threads=1

# Check all at once
make check
```

## 🔧 Troubleshooting

### Common Issues

#### Tests Failing with "InvalidPort(0)" Error
**Problem**: Tests are interfering with each other due to environment variable persistence.

**Solution**: Run tests sequentially:
```bash
cargo test -- --test-threads=1
```

#### Configuration Not Loading
**Problem**: Config file not found or environment variables not being read.

**Solutions**:
- Ensure `config.toml` exists in the project root
- Check environment variable naming: `APP_` prefix required
- Verify file permissions and syntax

#### Application Hanging on Startup
**Problem**: Server appears to start but doesn't show runtime output.

**Solutions**:
- Check if port is already in use: `lsof -i :3000`
- Verify configuration values are valid
- Run with debug logging: `RUST_LOG=debug cargo run -p api-gateway`

#### Environment Variables Not Working
**Problem**: Environment variables aren't overriding config file values.

**Known Limitation**: The `config` crate has limitations with complex types (arrays, maps) from environment variables.

**Solutions**:
- Use config files for complex configuration
- Use environment variables for simple values only
- See `config.dev.toml` for examples

### Debug Commands
```bash
# Check configuration loading
RUST_LOG=debug cargo run -p api-gateway

# Test specific configuration
cargo test -p api-gateway test_config_defaults_only -- --nocapture

# Check for port conflicts
netstat -tulpn | grep :3000

# Verify environment variables
env | grep APP_
```

### Getting Help
- Check the logs for detailed error messages
- Review `config.dev.toml` for configuration examples
- Run tests with `--nocapture` to see debug output
- Ensure all dependencies are installed: `cargo build`
