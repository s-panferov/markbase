use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::Index;

use std::path::Path;

pub struct SearchEngine {
  index: Index,
}

impl SearchEngine {
  pub fn new(storage: &Path) -> SearchEngine {
    let mut schema_builder = SchemaBuilder::default();
    schema_builder.add_text_field("key", TEXT | STORED);
    schema_builder.add_text_field("title", TEXT);
    schema_builder.add_text_field("body", TEXT);

    std::fs::create_dir_all(storage).expect("Unable to crate search folder");

    let dir = MmapDirectory::open(storage).expect("Unable to open search folder");

    let schema = schema_builder.build();
    let index = Index::open_or_create(dir, schema.clone()).unwrap();

    SearchEngine { index }
  }
}
