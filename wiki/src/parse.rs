use pulldown_cmark::Parser;
use pulldown_cmark::Event;

pub fn parse(source: &str) {
  let parser = Parser::new(source);
  parser.map(|ev| {
    match ev {
    }
  })
}