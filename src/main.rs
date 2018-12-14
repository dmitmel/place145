#[macro_use]
extern crate log;
extern crate env_logger;

extern crate failure;

extern crate actix;
extern crate actix_derive;
extern crate actix_web;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[macro_use]
mod macros;
mod api;
mod canvas;
mod websocket;

use failure::{Error, Fallible, ResultExt};

use actix::prelude::*;

use canvas::Canvas;

pub type State = Addr<Canvas>;

fn main() {
  run_fallible(|| {
    std::env::set_var("RUST_LOG", "info");
    env_logger::try_init().context("couldn't initialize logger")?;

    let system = actix::System::new("http-server");

    let canvas_addr = Arbiter::builder()
      .name("canvas")
      .stop_system_on_panic(true)
      .start(|_| run_fallible(Canvas::load));

    use actix_web::{fs, middleware, server, ws, App};
    let http_server = server::new(move || {
      App::with_state(canvas_addr.clone())
        .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
        .resource("/api/canvas", |r| r.with(api::canvas))
        .resource("/api/stream", |r| r.f(|req| ws::start(req, websocket::Ws)))
        .handler(
          "/",
          fs::StaticFiles::new("frontend/build")
            .unwrap()
            .index_file("index.html"),
        )
    });

    http_server
      .bind("0.0.0.0:8080")
      .context("couldn't bind server socket")?
      .start();

    let exit_code = system.run();
    std::process::exit(exit_code);
  })
}

pub fn run_fallible<T, F>(f: F) -> T
where
  F: FnOnce() -> Fallible<T>,
{
  f().unwrap_or_else(|error| {
    handle_error(error);
  })
}

pub fn handle_error(error: Error) -> ! {
  use std::{process, thread};

  let thread = thread::current();
  let name = thread.name().unwrap_or("<unnamed>");

  eprintln!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    eprintln!("caused by: {}", cause);
  }

  eprintln!("{}", error.backtrace());
  eprintln!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");

  process::exit(1);
}
