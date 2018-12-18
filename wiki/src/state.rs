use std::path::PathBuf;
use std::sync::Arc;

use crate::db::WikiBase;
use crate::log::Logger;

pub struct Context {
  pub root: PathBuf,
  pub base: WikiBase,
  pub log: Arc<Logger>,
}

pub struct AppState {
  pub ctx: Arc<Context>,
}
