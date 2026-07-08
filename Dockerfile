FROM rust:slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /usr/local/bin

COPY --from=builder /app/target/release/mailcheck /usr/local/bin/mailcheck

RUN chmod +x /usr/local/bin/mailcheck

WORKDIR /workspace

CMD ["/bin/bash"]
