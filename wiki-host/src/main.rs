#![feature(async_await)]
#![feature(futures_api)]

extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate url;

extern crate wiki_db;
extern crate wiki_log;
extern crate wiki_vm;

extern crate clap;

use std::path::PathBuf;
use std::sync::Arc;
use wiki_log::prelude::*;

mod cli;
mod conf;
mod state;
mod work;

fn main() {
  let log = wiki_log::init();
  let _guard = wiki_log::set_global_logger(log);

  let argv = cli::init();
  let input = argv
    .value_of("input")
    .expect("Input value should be defined");

  let storage = argv
    .value_of("storage")
    .expect("Storage value should be defined");

  let _cfg = conf::init();

  info!("Init wiki"; "folder" => input);
  // let watcher = watch::init(cwd);

  // we have to crate a "root" runtime
  let _vm = wiki_vm::Vm::new();
  let mut worker = work::WorkerPool::new();

  let storage = PathBuf::from(storage);
  let root = PathBuf::from(input);

  let base = wiki_db::WikiBase::new(&storage);
  let ctx = Arc::new(self::state::Context {
    root: root,
    base: base,
  });

  let rx = wiki_fs::scan(&ctx.root);

  for path in rx.iter() {
    if path.to_str().unwrap().ends_with(".mdx") {
      // FIXME
      futures::executor::block_on(worker.compile(ctx.clone(), path));
    }
  }

  println!("DROP");
}
