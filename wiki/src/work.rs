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
  type Context = actix::Context<Self>;
}

impl actix::Handler<WatchEvent> for WikiCompiler {
    type Result = Result<Option<Article>, BuildError>;

    fn handle(&mut self, msg: WatchEvent, _ctx: &mut actix::Context<Self>) -> Self::Result {
      let env = &self.env;

      match msg {
        WatchEvent::Update(file_name) => {
          if !file_name.to_str().unwrap().ends_with(".md") {
            return Ok(None)
          }

          debug!(env.log, "File updated"; "file" => file_name.to_str().unwrap());

          let key = Article::key(&env.root, &file_name);
          let article = env.base.get(&key);

          let source = std::str::from_utf8(std::fs::read(&file_name).unwrap().as_slice())
            .unwrap()
            .to_owned();

          let hash = Article::content_hash(&source);

          let article = match article {
            Some(mut article) => {
              if article.compiler_ver == crate::version::VERSION && article.hash == hash {
                debug!(env.log, "Article was not changed"; "file" => file_name.to_str().unwrap());
                return Ok(Some(article));
              }

              article.update((source, hash));
              article
            }
            None => Article::from_str(key, (source, hash)),
          };

          debug!(env.log, "Saving article {:?}", article);
          env.base.save(&article).unwrap();

          Ok(Some(article))
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
