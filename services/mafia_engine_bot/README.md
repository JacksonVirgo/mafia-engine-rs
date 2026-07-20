# Mafia Engine Bot

## Local database

Copy the repository's `.env.example` to `.env`, replace its placeholder secrets, then start MySQL:

```sh
docker compose up -d database
```

The bot constructs its MySQL URL from the `DATABASE_HOST`, `DATABASE_PORT`,
`DATABASE_NAME`, `DATABASE_USER`, and `DATABASE_PASSWORD` variables. It runs
pending migrations when it starts.
