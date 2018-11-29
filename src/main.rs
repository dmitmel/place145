#[macro_use]
extern crate log;
extern crate env_logger;

extern crate actix;
extern crate actix_web;

mod websocket;

fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();

  use actix_web::{fs, middleware, server, ws, App};

  server::new(|| {
    App::new()
      .middleware(middleware::Logger::default())
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
