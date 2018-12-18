use std::path::PathBuf;
use std::sync::Arc;

use ::futures::Future;
use failure::Fail;
use futures::task::SpawnExt;

use crate::article::Article;
use crate::log::prelude::*;
use crate::state::Context;

pub struct WorkerPool {
  pool: futures::executor::ThreadPool,
}

#[derive(Debug, Fail)]
pub enum BuildError {
  #[fail(display = "Error")]
  Error,
}

impl WorkerPool {
  pub fn new() -> WorkerPool {
    let mut pool = futures::executor::ThreadPoolBuilder::new();
    let pool = pool.create().unwrap();
    WorkerPool { pool }
  }

  pub fn compile(
    &mut self,
    ctx: Arc<Context>,
    file_name: PathBuf,
  ) -> impl Future<Output = Result<Article, BuildError>> {
    let future = async move {
      let key = Article::key(&ctx.root, &file_name);
      let article = ctx.base.get(&key);

      let source = std::str::from_utf8(std::fs::read(&file_name).unwrap().as_slice())
        .unwrap()
        .to_owned();

      let hash = Article::content_hash(&source);

      let article = match article {
        Some(mut article) => {
          if article.compiler_ver == crate::version::VERSION && article.hash == hash {
            debug!(ctx.log, "Article was not changed"; "file" => file_name.to_str().unwrap());
            return Ok(article);
          }

          article.update((source, hash));
          article
        }
        None => Article::from_str(key, (source, hash)),
      };

      debug!(ctx.log, "Saving article {:?}", article);
      ctx.base.save(&article).unwrap();

      Ok(article)
    };

    self.pool.spawn_with_handle(future).unwrap()
  }
}
