use canvas::Coord;

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub canvas: CanvasConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  pub address: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct CanvasConfig {
  pub width: Coord,
  pub height: Coord,
  pub save: CanvasSaveConfig,
}

#[derive(Debug, Deserialize)]
pub struct CanvasSaveConfig {
  pub interval: u64,
  pub path: PathBuf,
}

pub fn load(path: &Path) -> Config {
  let bytes = fs::read(path).unwrap();
  serde_json::from_slice(&bytes).unwrap()
}
