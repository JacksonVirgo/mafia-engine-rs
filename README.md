# mafia-engine-rs

The executable demonstrates framework-managed slash commands:

```rust
#[slash_command(description = "Send a greeting")]
async fn greet(
    context: CommandContext,
    #[description = "Who should be greeted?"] name: String,
    #[description = "How many greetings to send"] times: Option<i64>,
) -> Result<(), BoxError> {
    context.respond(format!("Hello, {name}! x{}", times.unwrap_or(1))).await
}

app.add_interaction(greet());
```

The macro generates the schema and parses the inline parameters. The framework
registers the command on `Ready`, routes its interaction by name, and leaves
your normal `on_ready` listener separate.

```sh
DISCORD_TOKEN=... cargo run
```

Invite the bot with both the `bot` and `applications.commands` OAuth2 scopes,
then run `/greet name:Jack times:2` in Discord.
