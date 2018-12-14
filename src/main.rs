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
use std::fs;
use std::path::Path;

use actix::prelude::*;
use actix_web::{middleware, server, ws, App};

use canvas::Canvas;
use config::Config;

pub type State = Addr<Canvas>;

fn main() {
  try_run(|| {
    env::set_var("RUST_LOG", "info,place145=debug");
    env_logger::try_init().context("couldn't initialize logger")?;

    let args: Vec<OsString> = env::args_os().collect();
    debug!("command line arguments: {:?}", args);

    let config_path = if args.len() == 2 {
      Path::new(&args[1])
    } else {
      let executable_path = &args[0];
      bail!("usage: {} path/to/config.json", executable_path.to_string_lossy());
    };

    info!("loading config from {:?}", config_path);
    let config_bytes =
      fs::read(config_path).context("couldn't read config file")?;
    let config = serde_json::from_slice(&config_bytes).unwrap();
    debug!("config loaded: {:#?}", config);

    let Config { server: server_config, canvas: canvas_config } = config;

    let system = actix::System::new("http-server");

    let canvas_addr = Arbiter::builder()
      .name("canvas")
      .stop_system_on_panic(true)
      .start(|_| try_run(|| Canvas::load(canvas_config)));

    let http_server = server::new(move || {
      App::with_state(canvas_addr.clone())
        .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
        .resource("/api/canvas", |r| r.with(api::canvas))
        .resource("/api/stream", |r| r.f(|req| ws::start(req, websocket::Ws)))
        .handler(
          "/",
          actix_web::fs::StaticFiles::new("frontend/build")
            .unwrap()
            .index_file("index.html"),
        )
    });

    info!("starting HTTP server");
    http_server
      .bind(server_config.address)
      .context("couldn't bind server socket")?
      .start();

    let exit_code = system.run();
    debug!("exiting with code {}", exit_code);
    std::process::exit(exit_code);
  })
}

pub fn try_run<T, F>(f: F) -> T
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

  error!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    error!("caused by: {}", cause);
  }

  error!("{}", error.backtrace());
  error!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");

  process::exit(1);
}
