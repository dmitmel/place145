use log::*;

#[macro_use]
mod macros;
mod api;
mod canvas;
mod client;
mod config;

use failure::{Error, Fallible, ResultExt};

use std::env;
use std::fs;
use std::path::PathBuf;

use actix::prelude::*;
use actix_web::{middleware, server, ws, App};

use self::canvas::Canvas;
use self::client::Client;
use self::config::*;

pub type State = Addr<Canvas>;

fn main() {
  try_run(|| {
    init_logger()?;

    let config_path = PathBuf::from("config.json");
    let config = load_config(config_path)?;

    let Config { server: server_config, canvas: canvas_config } = config;

    let system = System::new("http-server");
    let canvas_addr = start_canvas_actor(canvas_config);

    let static_files_config = server_config.static_files;
    let http_server = server::new(move || {
      App::with_state(canvas_addr.clone())
        .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
        .resource("/api/canvas", |r| r.with(api::canvas))
        .resource("/api/stream", |r| r.f(|req| ws::start(req, Client::new())))
        .handler(
          &static_files_config.base_url,
          actix_web::fs::StaticFiles::new(&static_files_config.path)
            .unwrap()
            .index_file(static_files_config.index_file.clone()),
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

fn init_logger() -> Fallible<()> {
  env::set_var("RUST_LOG", "info,place145=debug");
  env_logger::try_init().context("couldn't initialize logger")?;
  Ok(())
}

fn load_config(path: PathBuf) -> Fallible<Config> {
  info!("loading config from {:?}", path);
  let bytes: Vec<u8> = fs::read(path).context("couldn't read config file")?;
  let config: Config = serde_json::from_slice(&bytes).unwrap();
  debug!("config loaded: {:#?}", config);
  Ok(config)
}

fn start_canvas_actor(config: CanvasConfig) -> Addr<Canvas> {
  Arbiter::builder()
    .name("canvas")
    .stop_system_on_panic(true)
    .start(|_| try_run(|| Canvas::load(config)))
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
