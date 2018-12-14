use std::collections::HashSet;
use std::time::Duration;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use actix::prelude::*;

use websocket::Ws;

pub type Coord = u16;
pub type Color = u8;

const WIDTH: Coord = 64;
const HEIGHT: Coord = WIDTH;

const SAVE_INTERVAL: Duration = Duration::from_secs(2);
const FILE_PATH: &str = "canvas.bin";

#[derive(Debug)]
pub struct Canvas {
  file: File,
  data: Vec<Color>,
  listeners: HashSet<Addr<Ws>>,
}

impl Canvas {
  pub fn new() -> Self {
    let mut file: File =
      OpenOptions::new().read(true).write(true).open(FILE_PATH).unwrap();

    let mut data = vec![0u8; WIDTH as usize * HEIGHT as usize];
    file.read_exact(&mut data).unwrap();

    Self { file, data, listeners: HashSet::new() }
  }

  fn broadcast(&self, msg: CellUpdated) {
    for addr in &self.listeners {
      addr.do_send(msg.clone());
    }
  }

  fn assert_in_bounds(&self, x: Coord, y: Coord) -> Option<String> {
    macro_rules! is_in_bounds {
      ($val:expr, $val_name:expr, $max:expr, $max_name:expr) => {
        if $val >= $max {
          return Some(format!(
            "{} out of bounds: the {} is {} but the {} is {}",
            $val_name, $max_name, $max, $val_name, $val,
          ));
        }
      };
    }

    is_in_bounds!(x, "x", WIDTH, "width");
    is_in_bounds!(y, "y", HEIGHT, "height");

    None
  }

  fn cell_ref(&mut self, x: Coord, y: Coord) -> &mut Color {
    &mut self.data[x as usize + y as usize * WIDTH as usize]
  }
}

impl Actor for Canvas {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.run_interval(SAVE_INTERVAL, |actor, _ctx| {
      actor.file.seek(SeekFrom::Start(0)).unwrap();
      actor.file.set_len(0).unwrap();

      actor.file.write_all(&actor.data).unwrap();
      actor.file.flush().unwrap();
    });
  }
}

#[derive(Debug, Message)]
#[rtype("Result<Color, String>")]
pub struct GetCell {
  pub x: Coord,
  pub y: Coord,
}

actix_handler!(GetCell, Canvas, |self_, msg, _| {
  if let Some(err) = self_.assert_in_bounds(msg.x, msg.y) {
    return Err(err);
  }

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
  if let Some(err) = self_.assert_in_bounds(msg.x, msg.y) {
    return Err(err);
  }

  *self_.cell_ref(msg.x, msg.y) = msg.color;
  self_.broadcast(CellUpdated { x: msg.x, y: msg.y, color: msg.color });
  Ok(())
});

#[derive(Debug, Clone, Message)]
pub struct CellUpdated {
  pub x: Coord,
  pub y: Coord,
  pub color: Color,
}

#[derive(Debug, Message)]
pub struct ListenerConnected {
  pub addr: Addr<Ws>,
}

actix_handler!(ListenerConnected, Canvas, |self_, msg, _| {
  self_.listeners.insert(msg.addr);
});

#[derive(Debug, Message)]
pub struct ListenerDisconnected {
  pub addr: Addr<Ws>,
}

actix_handler!(ListenerDisconnected, Canvas, |self_, msg, _| {
  self_.listeners.remove(&msg.addr);
});
