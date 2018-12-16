#![feature(async_await)]
#![feature(futures_api)]

extern crate actix;
extern crate actix_web;
extern crate clap;
extern crate futures;
extern crate url;

extern crate wiki_db;
extern crate wiki_log;
extern crate wiki_vm;

use std::path::PathBuf;
use std::sync::Arc;

use wiki_log::prelude::*;

mod cli;
mod conf;
mod state;
mod work;

fn main() {
  let log = Arc::new(wiki_log::init());

  let argv = cli::init();
  let input = argv
    .value_of("input")
    .expect("Input value should be defined");

  let storage = argv
    .value_of("storage")
    .expect("Storage value should be defined");

  info!(log, "Starting wikibase"; "input" => input, "storage" => storage);

  match argv.subcommand_matches("reset") {
    Some(_) => {
      warn!(log, "Removing '{}'", storage);
      std::fs::remove_dir_all(storage).unwrap();
      return;
    }
    None => (),
  };

  let _cfg = conf::init();

  // we have to crate a "root" runtime
  let _vm = wiki_vm::Vm::new();
  let mut worker = work::WorkerPool::new();

  let storage = PathBuf::from(storage);
  let root = PathBuf::from(input);

  let base = wiki_db::WikiBase::new(&storage);
  let ctx = Arc::new(self::state::Context {
    root,
    base,
    log: log.clone(),
  });

  let thread = std::thread::spawn(move || {
    let rx = wiki_fs::scan(&ctx.root);
    for path in rx.iter() {
      if path.to_str().unwrap().ends_with(".mdx") {
        debug!(log, "Visit files"; "file" => path.to_str().unwrap());
        // FIXME blocking
        futures::executor::block_on(worker.compile(ctx.clone(), path));
      }
    }

    let rx = wiki_fs::watch(&ctx.root);
    for event in rx.iter() {
      match event {
        wiki_fs::WatchEvent::Update(path) => {
          if path.to_str().unwrap().ends_with(".mdx") {
            debug!(log, "File updated"; "file" => path.to_str().unwrap());
            // FIXME blocking
            futures::executor::block_on(worker.compile(ctx.clone(), path));
          }
        }
        wiki_fs::WatchEvent::Remove(path) => {
          debug!(log, "File removed"; "file" => path.to_str().unwrap());
        }
        wiki_fs::WatchEvent::Rename(from, to) => {
          debug!(log, "File renamed"; "file" => to.to_str().unwrap(), "from" => from.to_str().unwrap());
        }
      }
    }
  });

  thread.join().unwrap();
  println!("DROP");
}
