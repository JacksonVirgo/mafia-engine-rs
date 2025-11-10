pub enum LogFeature {
    Unknown,
    Signup,
}

impl LogFeature {
    pub fn to_string(&self) -> String {
        use LogFeature::*;
        let str = match self {
            Signup => "Signups",
            _ => "Other",
        };
        str.to_string()
    }
}
