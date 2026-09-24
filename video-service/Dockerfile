FROM rust:1.88-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY video-service ./video-service
COPY identity-service/Cargo.toml ./identity-service/Cargo.toml
COPY transcoding-worker/Cargo.toml ./transcoding-worker/Cargo.toml
RUN cargo build --release -p video-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app
COPY --from=build /src/target/release/video-service /usr/local/bin/video-service
USER 10001:10001
EXPOSE 8082
ENTRYPOINT ["/usr/local/bin/video-service"]
