use crate::prelude::*;

pub trait Plugin {
    fn build(&self, app: &mut AppBuilder);
}

#[macro_export]
macro_rules! plugin {
    ($name:ident, |$app_ident:ident| $body:block) => {
        pub struct $name;

        impl Plugin for $name {
            fn build(&self, app: &mut AppBuilder) {
                let $app_ident = app;
                $body
            }
        }
    };

    ($name:ident, $body:block) => {
        feature_plugin!($name, |app| $body);
    };
}
