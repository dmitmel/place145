#[macro_use]
extern crate log;
extern crate env_logger;

extern crate actix;
extern crate actix_web;

extern crate serde;
#[macro_use]
extern crate serde_derive;

mod api;
mod canvas;
mod websocket;

use std::sync::{Arc, Mutex};

use canvas::*;

pub type State = Arc<Mutex<Canvas>>;

fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();

  let state =
    Arc::new(Mutex::new(canvas::Canvas::new(10, 10, vec![0; 10 * 10])));

  use actix_web::{fs, middleware, server, ws, App};
  server::new(move || {
    App::with_state(state.clone())
      .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
      .resource("/api/cell", |r| {
        r.get().with(api::get_cell);
        r.put().with(api::update_cell);
      })
      .resource("/api/stream", |r| r.f(|req| ws::start(req, websocket::Ws)))
      .handler(
        "/",
        fs::StaticFiles::new("static").unwrap().index_file("index.html"),
      )
  })
  .bind("localhost:8080")
  .unwrap()
  .run();
}
