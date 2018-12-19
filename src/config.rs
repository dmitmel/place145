use serde_derive::*;

use crate::canvas::Coord;

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub canvas: CanvasConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub address: SocketAddr,
  pub static_files: ServerStaticFilesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStaticFilesConfig {
  pub path: PathBuf,
  pub base_url: String,
  pub index_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CanvasConfig {
  pub width: Coord,
  pub height: Coord,
  pub save: CanvasSaveConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CanvasSaveConfig {
  pub interval: u64,
  pub path: PathBuf,
}
