use crate::app::App;
use std::any::Any;

pub trait Plugin: Any + Send + Sync {
    fn build(&self, app: &mut App);

    fn ready(&self, _app: &App) -> bool {
        true
    }

    fn finish(&self, _app: &mut App) {}

    fn cleanup(&self, _app: &mut App) {}
}
