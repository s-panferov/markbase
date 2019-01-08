use actix_web::{server, App, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

use crate::state::AppState;

use typed_html::{dom::DOMTree, html, text};

fn index(req: &HttpRequest<Arc<AppState>>) -> impl Responder {
  let state = req.state();

  let html: DOMTree<String> = html!(
    <html id="hello-world">
      <head>
        <title>"Test"</title>
      </head>
      <body>
        {
          state.env.base.iter().map(|a| {
            println!("{:?}", a);
            let title = a.title;
            html!(
              <a>{text!("{}", title)}</a>
            )
          })
        }
      </body>
    </html>
  );

  HttpResponse::Ok()
    .content_type("text/html")
    .body(html.to_string())
}

pub fn start(state: Arc<AppState>) {
  server::new(move || App::with_state(state.clone()).resource("/", |r| r.f(index)))
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();
}
