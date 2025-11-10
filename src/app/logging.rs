use poise::serenity_prelude::{
    CreateAllowedMentions, CreateAttachment, ExecuteWebhook, Http, Webhook,
};
use tracing::{debug, error, info, warn};

pub enum LogType {
    Info,
    Warn,
    Error,
    Debug,
    Critical,
}

pub enum LogFeature {
    Unknown,
    Signup,
}

pub fn log<D: Into<String>>(log_type: LogType, str: impl Into<String>, data: Option<D>) {
    log_feature(log_type, LogFeature::Unknown, str, data);
}

pub fn log_feature<D: Into<String>>(
    log_type: LogType,
    _: LogFeature,
    str: impl Into<String>,
    data: Option<D>,
) {
    let message = str.into();

    #[cfg(debug_assertions)]
    log_cli(&log_type, &message);

    let Ok(webhook_url) = std::env::var("LOG_WEBHOOK") else {
        return;
    };

    let data = match data {
        Some(d) => Some(d.into().into_bytes()),
        _ => None,
    };

    tokio::spawn(async move {
        let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
        let http = Http::new(&token);

        let Ok(webhook) = Webhook::from_url(&http, webhook_url.as_str()).await else {
            return;
        };

        let level = match log_type {
            LogType::Debug => "DEBUG",
            LogType::Info => "INFO",
            LogType::Warn => "WARN",
            LogType::Error => "ERROR",
            LogType::Critical => "CRITICAL",
        };

        let content = format!("**{}**: {}", level, message);

        let mut builder = ExecuteWebhook::new()
            .content(content)
            .allowed_mentions(CreateAllowedMentions::new());

        if let Some(d) = data {
            let attachment = CreateAttachment::bytes(d, "log.txt");
            builder = builder.add_file(attachment);
        }

        match webhook.execute(&http, false, builder).await {
            Err(e) => error!("failed to send log webhook: {}", e),
            _ => {}
        }
    });
}

pub fn log_cli(log_type: &LogType, str: &String) {
    match log_type {
        LogType::Debug => debug!("{}", str),
        LogType::Info => info!("{}", str),
        LogType::Warn => warn!("{}", str),
        LogType::Error => error!("{}", str),
        LogType::Critical => error!("[CRITICAL] {}", str),
    }
}
