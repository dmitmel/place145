#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate failure;

extern crate actix;
extern crate actix_derive;
extern crate actix_web;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate serde_json;

#[macro_use]
mod macros;
mod api;
mod canvas;
mod config;
mod websocket;

use failure::{Error, Fallible, ResultExt};

use std::env;
use std::ffi::OsString;
use std::path::Path;

use actix::prelude::*;

use canvas::Canvas;
use config::Config;

pub type State = Addr<Canvas>;

fn main() {
  run_fallible(|| {
    env::set_var("RUST_LOG", "info");
    env_logger::try_init().context("couldn't initialize logger")?;

    let args: Vec<OsString> = env::args_os().collect();
    let config_path = if args.len() == 2 {
      Path::new(&args[1])
    } else {
      let executable_path = &args[0];
      bail!("usage: {} path/to/config.json", executable_path.to_string_lossy());
    };

    let config = config::load(config_path);
    let Config { server: server_config, canvas: canvas_config } = config;

    let system = actix::System::new("http-server");

    let canvas_addr = Arbiter::builder()
      .name("canvas")
      .stop_system_on_panic(true)
      .start(|_| run_fallible(|| Canvas::load(canvas_config)));

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
      .bind(server_config.address)
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
