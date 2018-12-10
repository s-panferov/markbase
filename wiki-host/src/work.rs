use std::cell::RefCell;
use std::sync::Arc;
use wiki_vm::{Compiler, ErrorInfo};

use std::fs::read;
use std::path::PathBuf;

use ::futures::Future;
use futures::task::SpawnExt;

use failure::Fail;

use crate::state::Context;
use wiki_db::Article;

thread_local! {
  pub static COMPILER: RefCell<Option<Compiler>> = RefCell::new(None);
}

pub struct WorkerPool {
  pool: futures::executor::ThreadPool,
}

#[derive(Debug, Fail)]
pub enum BuildError {
  #[fail(display = "JsError: {:?}", 0)]
  JsError(ErrorInfo),
}

impl WorkerPool {
  pub fn new() -> WorkerPool {
    let path = PathBuf::from(file!())
      .parent()
      .unwrap()
      .join("../../wiki-mdx/dist/index.js");

    let buf = read(path).unwrap();
    let src = std::str::from_utf8(&buf).unwrap().to_owned();

    let mut pool = futures::executor::ThreadPoolBuilder::new();
    let pool = pool
      .after_start(move |_| {
        COMPILER.with(|cp| {
          let compiler = wiki_vm::Compiler::new(&src);
          *cp.borrow_mut() = Some(compiler);
        })
      })
      .create()
      .unwrap();

    WorkerPool { pool }
  }

  pub fn compile(
    &mut self,
    ctx: Arc<Context>,
    file_name: PathBuf,
  ) -> impl Future<Output = Result<Article, BuildError>> {
    println!("Spawn {:?}", file_name);

    let future = async move {
      let key = Article::key(&ctx.root, &file_name);
      let mut article = ctx.base.get(&key);

      let source = std::str::from_utf8(std::fs::read(file_name).unwrap().as_slice())
        .unwrap()
        .to_owned();

      let hash = Article::content_hash(&source);

      let updated = match article.as_mut() {
        Some(mut article) => article.hash != hash,
        None => true,
      };

      if updated {
        let source = source.clone();
        let compiled = COMPILER.with(|cp| {
          let mut cp = cp.borrow_mut();
          cp.as_mut().unwrap().compile(&source)
        });

        match compiled {
          Ok(result) => match article.as_mut() {
            Some(mut article) => {
              article.compiled = Some(result);
              article.hash = hash;
            }
            None => article = Some(Article::from_str(key, source, Some(hash))),
          },
          Err(err) => return Err(BuildError::JsError(err)),
        };
      };

      let article = article.unwrap();
      ctx.base.save(&article);

      Ok(article)
    };

    self.pool.spawn_with_handle(future).unwrap()
  }
}
