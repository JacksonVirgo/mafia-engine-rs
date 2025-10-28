PROJECT_NAME := mafia_engine

install:
	cargo fetch
	cargo install sqlx-cli --no-default-features --features postgres
	sqlx database create

build:
	docker build -t mafia-engine .

migrate:
	sqlx migrate run

prepare:
	cargo sqlx prepare -- --lib

run:
	docker compose --profile dev up --watch

run-prod:
	docker compose --profile prod up

test:
	cargo test

.PHONY: install build migrate prepare run run-prod test
