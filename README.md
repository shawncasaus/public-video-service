/public-video-service
  Cargo.toml                  # workspace manifest
  /crates
    api-gateway/              # control plane API (axum)
      Cargo.toml
      src/main.rs
    origin/                   # static file server (axum)
      Cargo.toml
      src/main.rs
    ingest-rtmp/              # nginx-rtmp wrapper or Rust ingest stub
      Dockerfile
      nginx.conf
    packager-hls/             # ffmpeg adapter (Day-1) or Rust packager later
      Cargo.toml
      src/main.rs
  /deploy/docker/
    docker-compose.yml
    api-gateway.Dockerfile
    origin.Dockerfile
    rtmp.Dockerfile
  /examples/player-hls/
    index.html     
