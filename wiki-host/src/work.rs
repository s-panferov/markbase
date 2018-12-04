use rayon::ThreadPool;
use std::cell::RefCell;
use wiki_vm::{Compiler, ErrorInfo};

use std::fs::read;
use std::path::PathBuf;

thread_local! {
  pub static COMPILER: RefCell<Option<Compiler>> = RefCell::new(None);
}

pub struct WorkerPool {
  pool: ThreadPool,
}

impl WorkerPool {
  pub fn new() -> WorkerPool {
    let path = PathBuf::from(file!())
      .parent()
      .unwrap()
      .join("../../wiki-mdx/dist/index.js");

    let buf = read(path).unwrap();
    let src = std::str::from_utf8(&buf).unwrap().to_owned();
    let pool = rayon::ThreadPoolBuilder::new()
      .start_handler(move |_| {
        COMPILER.with(|cp| {
          let compiler = wiki_vm::Compiler::new(&src);
          *cp.borrow_mut() = Some(compiler);
        })
      })
      .build()
      .unwrap();

    WorkerPool { pool }
  }

  pub fn compile(&self, path: PathBuf) -> Result<String, ErrorInfo> {
    self.pool.install(move || {
      let source = std::str::from_utf8(std::fs::read(path).unwrap().as_slice())
        .unwrap()
        .to_owned();

      COMPILER.with(|cp| {
        let mut cp = cp.borrow_mut();
        cp.as_mut().unwrap().compile(source)
      })
    })
  }
}
