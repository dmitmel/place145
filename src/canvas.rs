use std::collections::HashSet;

use actix::prelude::*;

use websocket::Ws;

pub type Coord = u16;
pub type Color = u8;

#[derive(Debug)]
pub struct Canvas {
  pub width: Coord,
  pub height: Coord,
  data: Vec<Color>,
  listeners: HashSet<Addr<Ws>>,
}

impl Actor for Canvas {
  type Context = Context<Self>;
}

impl Canvas {
  pub fn new(width: Coord, height: Coord) -> Self {
    Self {
      width,
      height,
      data: vec![0u8; width as usize * height as usize],
      listeners: HashSet::new(),
    }
  }

  fn broadcast(&self, msg: CellUpdated) {
    for addr in &self.listeners {
      addr.do_send(msg.clone());
    }
  }

  fn assert_in_bounds(&self, x: Coord, y: Coord) -> Option<String> {
    let w = self.width;
    let h = self.height;

    macro_rules! err {
      ($cond:expr, $($arg:tt)+) => {
        if $cond { return Some(format!($($arg)+)); }
      };
    }

    err!(x >= w, "x out of bounds: the width is {} but the x is {}", w, x);
    err!(y >= h, "y out of bounds: the height is {} but the y is {}", h, y);

    None
  }

  fn index(&self, x: Coord, y: Coord) -> usize {
    x as usize + y as usize * self.width as usize
  }
}

#[derive(Debug, Deserialize, Message)]
#[rtype("Result<Color, String>")]
pub struct GetCell {
  pub x: Coord,
  pub y: Coord,
}

actix_handler!(GetCell, Canvas, |self_, msg, _| {
  if let Some(err) = self_.assert_in_bounds(msg.x, msg.y) {
    return Err(err);
  }

  let i = self_.index(msg.x, msg.y);
  Ok(self_.data[i])
});

#[derive(Debug, Message)]
#[rtype("Vec<Color>")]
pub struct GetCanvas;

actix_handler!(GetCanvas, Canvas, |self_, _, _| self_.data.clone());

#[derive(Debug, Deserialize, Message)]
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

  let i = self_.index(msg.x, msg.y);
  self_.data[i] = msg.color;
  self_.broadcast(CellUpdated { x: msg.x, y: msg.y, color: msg.color });
  Ok(())
});

#[derive(Debug, Clone, Message, Serialize)]
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
