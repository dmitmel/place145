use log::*;

use failure::{Fallible, ResultExt};
use std::collections::HashSet;
use std::time::Duration;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use actix::actors::signal::{self, ProcessSignals, Signal, SignalType};
use actix::prelude::*;
use futures::Future;

use crate::config::CanvasConfig;
use crate::try_run;
use crate::websocket::Client;

pub type Coord = u16;
pub type Color = u8;

#[derive(Debug)]
pub struct Canvas {
  config: CanvasConfig,
  file: File,
  data: Vec<Color>,
  listeners: HashSet<Addr<Client>>,
}

impl Canvas {
  pub fn load(config: CanvasConfig) -> Fallible<Self> {
    info!("loading canvas data from {:?}", config.save.path);

    let mut file: File = OpenOptions::new()
      .read(true)
      .write(true)
      .open(&config.save.path)
      .context("couldn't open canvas file")?;

    let mut data = vec![0u8; config.width as usize * config.height as usize];
    file.read_exact(&mut data).context("couldn't read canvas data")?;

    Ok(Self { config, file, data, listeners: HashSet::new() })
  }

  fn assert_in_bounds(&self, x: Coord, y: Coord) -> Result<(), String> {
    macro_rules! is_in_bounds {
      ($val:expr, $val_name:expr, $max:expr, $max_name:expr) => {
        if $val >= $max {
          return Err(format!(
            "{} out of bounds: the {} is {} but the {} is {}",
            $val_name, $max_name, $max, $val_name, $val,
          ));
        }
      };
    }

    is_in_bounds!(x, "x", self.config.width, "width");
    is_in_bounds!(y, "y", self.config.height, "height");

    Ok(())
  }

  fn cell_ref(&mut self, x: Coord, y: Coord) -> &mut Color {
    &mut self.data[x as usize + y as usize * self.config.width as usize]
  }
}

impl Actor for Canvas {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_to_signals(ctx);

    let save_interval = Duration::from_millis(self.config.save.interval);
    ctx.run_interval(save_interval, |self_, _ctx| try_run(|| self_.save()));
  }
}

impl Canvas {
  fn subscribe_to_signals(&mut self, ctx: &mut <Canvas as Actor>::Context) {
    let service_addr = System::current().registry().get::<ProcessSignals>();

    service_addr
      .send(signal::Subscribe(ctx.address().recipient()))
      .map_err(|send_error| panic!(send_error))
      .into_actor(self)
      .wait(ctx);
  }

  fn save(&self) -> Fallible<()> {
    debug!("saving canvas data");

    let mut file = &self.file;
    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();

    file.write_all(&self.data).context("couldn't write canvas data")?;
    file.flush().context("couldn't flush canvas file")?;

    Ok(())
  }
}

actix_handler!(Signal, Canvas, |self_, msg, _| match msg.0 {
  SignalType::Int | SignalType::Term | SignalType::Quit => {
    try_run(|| self_.save())
  }
  _ => (),
});

#[derive(Debug, Message)]
#[rtype("Result<Color, String>")]
pub struct GetCell {
  pub x: Coord,
  pub y: Coord,
}

actix_handler!(GetCell, Canvas, |self_, msg, _| {
  self_.assert_in_bounds(msg.x, msg.y)?;
  Ok(*self_.cell_ref(msg.x, msg.y))
});

#[derive(Debug, Message)]
#[rtype("Vec<Color>")]
pub struct GetCanvas;

actix_handler!(GetCanvas, Canvas, |self_, _, _| self_.data.clone());

#[derive(Debug, Message)]
#[rtype("Result<(), String>")]
pub struct UpdateCell {
  pub x: Coord,
  pub y: Coord,
  pub color: Color,
}

actix_handler!(UpdateCell, Canvas, |self_, msg, _| {
  self_.assert_in_bounds(msg.x, msg.y)?;
  *self_.cell_ref(msg.x, msg.y) = msg.color;

  for addr in &self_.listeners {
    addr.do_send(CellUpdated { x: msg.x, y: msg.y, color: msg.color });
  }

  Ok(())
});

#[derive(Debug, Message)]
pub struct CellUpdated {
  pub x: Coord,
  pub y: Coord,
  pub color: Color,
}

#[derive(Debug, Message)]
pub struct ListenerConnected {
  pub addr: Addr<Client>,
}

actix_handler!(ListenerConnected, Canvas, |self_, msg, _| {
  self_.listeners.insert(msg.addr);
});

#[derive(Debug, Message)]
pub struct ListenerDisconnected {
  pub addr: Addr<Client>,
}

actix_handler!(ListenerDisconnected, Canvas, |self_, msg, _| {
  self_.listeners.remove(&msg.addr);
});
