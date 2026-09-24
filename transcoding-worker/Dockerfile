FROM rust:1.88-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY transcoding-worker ./transcoding-worker
COPY identity-service/Cargo.toml ./identity-service/Cargo.toml
COPY video-service/Cargo.toml ./video-service/Cargo.toml
RUN cargo build --release -p transcoding-worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates ffmpeg && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app
COPY --from=build /src/target/release/transcoding-worker /usr/local/bin/transcoding-worker
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/transcoding-worker"]
