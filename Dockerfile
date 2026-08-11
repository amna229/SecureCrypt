FROM rust:1.88 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/application .
COPY simplified-pki ./simplified-pki
COPY src/dashboard/ui ./src/dashboard/ui
COPY src/application/ui ./src/application/ui
CMD ["./tfg_project"]