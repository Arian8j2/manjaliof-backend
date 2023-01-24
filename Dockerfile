FROM rust:1.66.1-slim-bullseye AS builder
WORKDIR /builder
COPY . .
RUN apt-get -y update && apt-get -y install pkg-config libssl-dev
RUN cargo build --release

FROM debian:11-slim
WORKDIR /app
RUN apt-get -y update && apt-get -y install ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /builder/target/release/manjaliof-backend /app/manjaliof-backend
COPY Rocket.toml .
CMD ["./manjaliof-backend"]
