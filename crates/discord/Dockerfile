FROM rust:latest AS builder
WORKDIR /app

COPY . .

RUN cargo build --release -p discord

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/discord /usr/local/bin/discord
CMD ["discord"]
