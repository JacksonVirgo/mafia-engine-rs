pub enum ServerFlag {
    SuperServer,
    Signups,
    VoteCounter,
    PlayerChats,
    Archiving,
}

impl ServerFlag {
    pub fn to_string(&self) -> &'static str {
        match self {
            ServerFlag::SuperServer => "super",
            ServerFlag::Signups => "signups",
            ServerFlag::VoteCounter => "votecounter",
            ServerFlag::PlayerChats => "playerchats",
            ServerFlag::Archiving => "archiving",
        }
    }
}
