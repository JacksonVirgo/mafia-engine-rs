FROM rust:latest AS builder
WORKDIR /app

COPY . .

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

RUN cargo build --release -p archiving

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/archiving /usr/local/bin/archiving
CMD ["archiving"]

