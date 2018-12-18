use actix_web::{server, App, HttpRequest, Responder};
use std::sync::Arc;

use crate::state::AppState;

fn index(req: &HttpRequest<Arc<AppState>>) -> impl Responder {
  let state = req.state();
  for article in state.ctx.base.iter() {
    println!("Article {:?}", article);
  }
  "Hello"
}

pub fn start(state: Arc<AppState>) {
  server::new(move || App::with_state(state.clone()).resource("/", |r| r.f(index)))
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}
