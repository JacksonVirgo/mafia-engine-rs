pub type BotError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Rejection {
    pub message: String,
    pub ephemeral: bool,
}

impl Rejection {
    pub fn new(m: impl Into<String>) -> Self {
        Self {
            message: m.into(),
            ephemeral: true,
        }
    }

    pub fn public(m: impl Into<String>) -> Self {
        Self {
            message: m.into(),
            ephemeral: false,
        }
    }
}
