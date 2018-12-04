extern crate rocksdb;
extern crate tantivy;

use rocksdb::DB;
use std::path::PathBuf;

mod search;

use self::search::SearchEngine;

pub struct WikiBase {
  db: DB,
  search: SearchEngine,
}

impl WikiBase {
  pub fn new(storage: PathBuf) -> WikiBase {
    let db = DB::open_default(&storage.join("rocksdb")).unwrap();
    let search = SearchEngine::new(&storage.join("tantivy"));
    WikiBase { db, search }
  }
}
