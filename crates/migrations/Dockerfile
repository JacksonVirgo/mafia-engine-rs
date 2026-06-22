FROM rust:latest AS builder
WORKDIR /app

COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release -p migrations

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/migrations /usr/local/bin/migrations
CMD ["migrations"]
