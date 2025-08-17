
# Project structure

/public-video-service
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

