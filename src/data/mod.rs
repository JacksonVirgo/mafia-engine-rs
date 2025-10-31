use serde::{Deserialize, Serialize};

pub mod signups;
pub mod users;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqlBool(pub bool);

impl From<i8> for SqlBool {
    fn from(value: i8) -> Self {
        match value {
            0 => Self(false),
            1 => Self(true),
            _ => panic!("invalid boolean value {value}"),
        }
    }
}

pub mod prelude {
    pub use super::{
        signups::categories::*, signups::signups::*, signups::slots::*, signups::users::*, users::*,
    };
}
