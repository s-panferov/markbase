use std::path::PathBuf;
use std::sync::Arc;

use wiki_db::WikiBase;
use wiki_log::Logger;

pub struct Context {
  pub root: PathBuf,
  pub base: WikiBase,
  pub log: Arc<Logger>,
}

struct AppState {
  ctx: Arc<Context>,
}
