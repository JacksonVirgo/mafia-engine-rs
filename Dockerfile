ARG RUST_VERSION=1.89.0
ARG APP_NAME=mafia-engine-rs

FROM rust:${RUST_VERSION}-alpine AS build

RUN apk add \
    musl-dev \
    bash \
    build-base \
    pkgconf \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

COPY ./Cargo.lock ./
COPY ./Cargo.toml ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./.sqlx ./.sqlx

RUN cargo install sqlx-cli --no-default-features --features mysql

RUN cargo build --release

CMD ["sh", "-c", "sqlx migrate run && ./target/release/mafia-engine-rs"]
