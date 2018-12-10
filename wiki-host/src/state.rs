use std::path::PathBuf;
use std::sync::Arc;
use wiki_db::WikiBase;

pub struct Context {
  pub root: PathBuf,
  pub base: WikiBase,
}

struct AppState {
  ctx: Arc<Context>,
}
