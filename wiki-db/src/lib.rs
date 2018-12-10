extern crate rocksdb;
extern crate serde_yaml;
extern crate sha2;
extern crate tantivy;

#[macro_use]
extern crate serde_derive;

use rocksdb::DB;
use std::path::Path;

pub use self::article::{Article, ArticleKey};

mod article;
mod search;

use self::search::SearchEngine;

pub struct WikiBase {
  db: DB,
  search: SearchEngine,
}

impl WikiBase {
  pub fn new(storage: &Path) -> WikiBase {
    let db = DB::open_default(&storage.join("rocksdb")).unwrap();
    let search = SearchEngine::new(&storage.join("tantivy"));
    WikiBase { db, search }
  }

  pub fn get(&self, key: &ArticleKey) -> Option<Article> {
    self
      .db
      .get(key.as_bytes())
      .unwrap()
      .map(|b| serde_json::from_slice(&*b).unwrap())
  }

  pub fn save(&self, article: &Article) -> Result<(), rocksdb::Error> {
    self.db.put(
      article.key.as_bytes(),
      serde_json::to_vec(&article).unwrap().as_slice(),
    )
  }
}
