use crate::prelude::*;
use std::time::Instant;

#[poise::command(slash_command)]
pub async fn heartbeat(ctx: BotCtx<'_>) -> Result<(), BotError> {
    let server = ext::<db::Server>(ctx).await?;
    let uptime = ctx.data().started_at.elapsed();
    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    let start = Instant::now();
    let reply = ctx.say("Checking heartbeat...").await?;
    let latency_ms = start.elapsed().as_millis();

    reply
        .edit(
            ctx,
            poise::CreateReply::default().content(format!(
                "Server ID: {}\nLatency: {}ms\nUptime: {}d {:02}h {:02}m {:02}s",
                server.server_id, latency_ms, days, hours, minutes, seconds
            )),
        )
        .await?;

    Ok(())
}
