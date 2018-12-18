#![feature(async_await)]
#![feature(futures_api)]

#[macro_use]
extern crate serde_derive;

use std::path::PathBuf;
use std::sync::Arc;

use crate::log::prelude::*;

mod article;
mod cli;
mod conf;
mod db;
mod fs;
mod log;
mod search;
mod server;
mod state;
mod version;
mod work;

fn main() {
  let log = Arc::new(crate::log::init());

  let argv = cli::init();
  let input = argv
    .value_of("input")
    .expect("Input value should be defined");

  let storage = argv
    .value_of("storage")
    .expect("Storage value should be defined");

  std::fs::create_dir_all(&storage).unwrap();

  let input = std::fs::canonicalize(input).unwrap();
  let storage = std::fs::canonicalize(storage).unwrap();

  info!(log, "Starting wikibase"; "input" => input.to_str().unwrap(), "storage" => storage.to_str().unwrap());

  match argv.subcommand_matches("reset") {
    Some(_) => {
      warn!(log, "Removing '{}'", storage.to_str().unwrap());
      std::fs::remove_dir_all(storage).unwrap();
      return;
    }
    None => (),
  };

  let _cfg = conf::init();

  let mut worker = work::WorkerPool::new();

  let storage = PathBuf::from(storage);
  let root = PathBuf::from(input);

  let base = crate::db::WikiBase::new(&storage);
  let ctx = Arc::new(self::state::Context {
    root,
    base,
    log: log.clone(),
  });

  std::thread::spawn({
    let ctx = ctx.clone();
    move || {
      let rx = crate::fs::scan(&ctx.root);
      for path in rx.iter() {
        if path.to_str().unwrap().ends_with(".mdx") {
          debug!(log, "Visit files"; "file" => path.to_str().unwrap());
          // FIXME blocking
          futures::executor::block_on(worker.compile(ctx.clone(), path));
        }
      }

      let rx = crate::fs::watch(&ctx.root);
      for event in rx.iter() {
        match event {
          crate::fs::WatchEvent::Update(path) => {
            if path.to_str().unwrap().ends_with(".mdx") {
              debug!(log, "File updated"; "file" => path.to_str().unwrap());
              // FIXME blocking
              futures::executor::block_on(worker.compile(ctx.clone(), path));
            }
          }
          crate::fs::WatchEvent::Remove(path) => {
            debug!(log, "File removed"; "file" => path.to_str().unwrap());
          }
          crate::fs::WatchEvent::Rename(from, to) => {
            debug!(log, "File renamed"; "file" => to.to_str().unwrap(), "from" => from.to_str().unwrap());
          }
        }
      }
    }
  });

  let state = Arc::new(crate::state::AppState { ctx });
  server::start(state);
}
