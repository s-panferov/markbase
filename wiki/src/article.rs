use std::path::PathBuf;
use std::path::Path;
use std::str::FromStr;

use lazy_static::{lazy_static};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleKey(String);

impl ArticleKey {
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }

  pub fn file_name(&self) -> String {
    PathBuf::from_str(&self.0).unwrap().file_name().unwrap().to_str().unwrap().to_owned()
  }
}

lazy_static! {
  static ref TITLE_KEY: serde_yaml::Value = serde_yaml::Value::String(String::from("value"));
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Article {
  pub key: ArticleKey,
  pub title: String,
  pub source: String,
  pub compiler_ver: u32,
  pub compiled: Option<String>,
  pub hash: ArticleHash,
  pub meta: serde_yaml::Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleHash(Vec<u8>);

impl Article {
  pub fn from_str(key: ArticleKey, body: (String, ArticleHash)) -> Article {
    let (source, meta) = Self::parse_meta(body.0);
    let hash = body.1;

    let title = Self::get_title(&meta, key.file_name());

    Article {
      key,
      title,
      compiled: Self::parse_source(&source),
      compiler_ver: crate::version::VERSION,
      source,
      hash,
      meta,
    }
  }

  pub fn key(root: &Path, file_name: &Path) -> ArticleKey {
    let key = file_name
      .strip_prefix(&root)
      .expect("File should be inside root foolder")
      .to_str()
      .unwrap();

    ArticleKey(key.to_string())
  }

  pub fn content_hash(body: &str) -> ArticleHash {
    let mut hasher = Sha256::new();
    hasher.input(body.as_bytes());
    ArticleHash(hasher.result().as_slice().to_vec())
  }

  fn get_title(meta: &serde_yaml::Mapping, file_name: String) -> String {
    meta
      .get(&TITLE_KEY)
      .and_then(|s| s.as_str())
      .map(|s| s.to_string())
      .unwrap_or(file_name)
  }

  fn parse_meta(source: String) -> (String, serde_yaml::Mapping) {
    if source.starts_with("---") {
      let end = source[3..]
        .find("---")
        .expect("File should contain end for a \"---\" YAML block");

      let mapping =
        serde_yaml::from_str::<serde_yaml::Mapping>(&source[3..end]).expect("Should be parsed");
      (source[end..].to_string(), mapping)
    } else {
      (source, serde_yaml::Mapping::new())
    }
  }

  fn parse_source(source: &str) -> Option<String> {
    let mut compiled = String::new();
    pulldown_cmark::html::push_html(&mut compiled, pulldown_cmark::Parser::new(source));
    Some(compiled)
  }
}
