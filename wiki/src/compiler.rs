use std::path::PathBuf;
use std::sync::Arc;

use failure::Fail;

use crate::article::Article;
use crate::log::prelude::*;
use crate::state::WikiEnv;
use crate::fs::WatchEvent;

impl actix::Message for WatchEvent {
  type Result = Result<Option<Article>, BuildError>;
}

pub struct WikiCompiler {
  pub env: Arc<WikiEnv>
}

impl actix::Actor for WikiCompiler {
  type Context = actix::SyncContext<Self>;
}

impl WikiCompiler {
  pub fn compile(&self, file_name: PathBuf) -> Result<Option<Article>, BuildError> {
    let env = &self.env;
    let str_path = file_name.to_str().unwrap();
    if !str_path.ends_with(".md") {
      return Ok(None)
    }

    debug!(env.log, "File updated"; "file" => str_path);

    let key = Article::key(&env.root, &file_name);
    let article = env.base.get(&key);

    let source = std::str::from_utf8(std::fs::read(&file_name).unwrap().as_slice())
      .unwrap()
      .to_owned();

    let hash = Article::content_hash(&source);

    let article = match article {
      Some(article) => {
        if article.compiler_ver == crate::version::VERSION && article.hash == hash {
          debug!(env.log, "Article was not changed"; "file" => str_path);
          return Ok(Some(article));
        }

        Article::from_str(key, (source, hash))
      }
      None => Article::from_str(key, (source, hash)),
    };

    debug!(env.log, "Saving article {:?}", article);
    env.base.save(&article).unwrap();

    Ok(Some(article))
  }
}

impl actix::Handler<WatchEvent> for WikiCompiler {
    type Result = Result<Option<Article>, BuildError>;

    fn handle(&mut self, msg: WatchEvent, _ctx: &mut actix::SyncContext<Self>) -> Self::Result {
      match msg {
        WatchEvent::Update(file_name) => {
          self.compile(file_name)
        },
        WatchEvent::Remove(_) => {
          Err(BuildError::Error)
        },
        WatchEvent::Rename(_,_) => {
          Err(BuildError::Error)
        }
      }
    }
}

#[derive(Debug, Fail)]
pub enum BuildError {
  #[fail(display = "Error")]
  Error,
}
