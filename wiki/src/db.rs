extern crate rocksdb;
extern crate serde_yaml;
extern crate sha2;
extern crate tantivy;

use failure::Fail;
use rocksdb::DB;
use std::path::Path;

pub use crate::article::{Article, ArticleKey};
use crate::search::SearchEngine;

#[derive(Debug, Fail)]
pub enum BaseError {
  #[fail(display = "StorageError: {:?}", 0)]
  StorageError(rocksdb::Error),
}

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

  pub fn iter(&self) -> impl Iterator<Item = Article> {
    let iter = self.db.iterator(rocksdb::IteratorMode::Start);
    iter.map(|a| serde_json::from_slice(&*a.1).unwrap())
  }

  pub fn get(&self, key: &ArticleKey) -> Option<Article> {
    self
      .db
      .get(key.as_bytes())
      .unwrap()
      .map(|b| serde_json::from_slice(&*b).unwrap())
  }

  pub fn save(&self, article: &Article) -> Result<(), BaseError> {
    self
      .db
      .put(
        article.key.as_bytes(),
        serde_json::to_vec(&article).unwrap().as_slice(),
      )
      .map_err(BaseError::StorageError)
  }
}
