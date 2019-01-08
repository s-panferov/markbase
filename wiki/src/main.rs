#![feature(async_await)]
#![feature(futures_api)]
#![feature(proc_macro_hygiene)]
#![recursion_limit="128"]

#[macro_use]
extern crate serde_derive;

use std::path::PathBuf;
use std::sync::Arc;

use actix::prelude::*;
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
mod compiler;

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

  let base = crate::db::WikiBase::new(&storage);
  let root = PathBuf::from(input);

  let env = Arc::new(self::state::WikiEnv {
    root: root.clone(),
    base,
    log: log.clone(),
  });

  let code = actix::System::run(move || {
    let env2 = env.clone();
    let worker = actix::sync::SyncArbiter::start(2, move || compiler::WikiCompiler{ env: env2.clone() });
    let fs = fs::WikiWatch{folder: root, rx: worker.recipient()}.start();
    let state = Arc::new(crate::state::AppState { env });
    server::start(state);
  });

  std::process::exit(code);
}
