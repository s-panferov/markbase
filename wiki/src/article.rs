use std::path::Path;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleKey(String);

impl ArticleKey {
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }
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

  pub fn update(&mut self, source: (String, ArticleHash)) {
    self.hash = source.1;
    let (source, meta) = Self::parse_meta(source.0);
    self.compiled = Self::parse_source(&source);
    self.source = source;
    self.meta = meta;
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

  pub fn from_str(key: ArticleKey, body: (String, ArticleHash)) -> Article {
    let title_key = serde_yaml::Value::String("value".to_string());
    let (source, meta) = Self::parse_meta(body.0);
    let title = meta
      .get(&title_key)
      .and_then(|s| s.as_str())
      .map(|s| s.to_string())
      .unwrap_or("".to_string());

    let hash = body.1;

    Article {
      key: key,
      title: title,
      compiled: Self::parse_source(&source),
      compiler_ver: crate::version::VERSION,
      source,
      hash,
      meta,
    }
  }
}
