#[macro_use]
extern crate log;
extern crate env_logger;

extern crate actix;
extern crate actix_derive;
extern crate actix_web;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
mod macros;

mod api;
mod canvas;
mod websocket;

use actix::prelude::*;

use canvas::{Canvas, GetCell, UpdateCell};

pub type State = Addr<Canvas>;

fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();

  let system = actix::System::new("http-server");

  let canvas_addr = Arbiter::builder()
    .name("canvas")
    .stop_system_on_panic(true)
    .start(|_| Canvas::new(10, 10));

  use actix_web::{fs, middleware, server, ws, App};
  server::new(move || {
    App::with_state(canvas_addr.clone())
      .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
      .resource("/api/cell", |r| {
        r.get().with(|req| {
          api::canvas_message::<GetCell, _, _>(req, |color| color.to_string())
        });
        r.put().with(|req| {
          api::canvas_message::<UpdateCell, _, _>(req, |_| "".to_string())
        });
      })
      .resource("/api/stream", |r| r.f(|req| ws::start(req, websocket::Ws)))
      .handler(
        "/",
        fs::StaticFiles::new("static").unwrap().index_file("index.html"),
      )
  })
  .bind("localhost:8080")
  .unwrap()
  .start();

  let exit_code = system.run();
  std::process::exit(exit_code);
}
