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
  pub body: String,
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

  pub fn from_str(key: ArticleKey, body: String, hash: Option<ArticleHash>) -> Article {
    let meta = if body.starts_with("---") {
      let end = body[3..]
        .find("---")
        .expect("File should contain end for a \"---\" YAML block");

      serde_yaml::from_str::<serde_yaml::Mapping>(&body[3..end]).expect("Should be parsed")
    } else {
      serde_yaml::Mapping::new()
    };

    let title_key = serde_yaml::Value::String("value".to_string());
    let title = meta
      .get(&title_key)
      .and_then(|s| s.as_str())
      .map(|s| s.to_string())
      .unwrap_or("".to_string());

    let hash = hash.unwrap_or_else(|| Article::content_hash(&body));

    Article {
      key: key,
      title: title,
      compiled: None,
      body,
      hash,
      meta,
    }
  }
}
