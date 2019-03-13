use log::*;

#[macro_use]
mod macros;
mod canvas;
mod client;
mod config;
mod server;

use failure::{Error, Fallible, ResultExt};

use std::env;
use std::fs;
use std::path::PathBuf;

use actix::prelude::*;

use self::canvas::Canvas;
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
    server::start(server_config, canvas_addr)?;

    let exit_code = system.run();
    debug!("exiting with code {}", exit_code);
    std::process::exit(exit_code);
  })
}

fn init_logger() -> Fallible<()> {
  if let Err(env::VarError::NotPresent) = env::var("RUST_LOG") {
    env::set_var("RUST_LOG", "info,place145=debug");
  }
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
