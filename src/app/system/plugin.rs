use crate::app::system::app_builder::AppBuilder;

pub trait Plugin {
    fn build(&self, app: &mut AppBuilder);
}
