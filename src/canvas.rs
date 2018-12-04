use std::collections::HashSet;

use actix::prelude::*;

use websocket::Ws;

pub type Color = u8;

#[derive(Debug)]
pub struct Canvas {
  pub width: usize,
  pub height: usize,
  data: Vec<Color>,
  listeners: HashSet<Addr<Ws>>,
}

impl Actor for Canvas {
  type Context = Context<Self>;
}

impl Canvas {
  pub fn new(width: usize, height: usize) -> Self {
    Self {
      width,
      height,
      data: vec![0; width * height],
      listeners: HashSet::new(),
    }
  }

  fn broadcast(&self, msg: CellUpdated) {
    for addr in &self.listeners {
      addr.do_send(msg.clone());
    }
  }

  fn assert_in_bounds(&self, x: usize, y: usize) -> Option<String> {
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
}

#[derive(Debug, Deserialize, Message)]
#[rtype("Result<u8, String>")]
pub struct GetCell {
  pub x: usize,
  pub y: usize,
}

actix_handler!(GetCell, Canvas, |self_, msg, _| {
  if let Some(err) = self_.assert_in_bounds(msg.x, msg.y) {
    return Err(err);
  }

  Ok(self_.data[msg.y * self_.width + msg.x])
});

#[derive(Debug, Deserialize, Message)]
#[rtype("Result<(), String>")]
pub struct UpdateCell {
  pub x: usize,
  pub y: usize,
  pub color: u8,
}

actix_handler!(UpdateCell, Canvas, |self_, msg, _| {
  if let Some(err) = self_.assert_in_bounds(msg.x, msg.y) {
    return Err(err);
  }

  self_.data[msg.y * self_.width + msg.x] = msg.color;
  self_.broadcast(CellUpdated { x: msg.x, y: msg.y, color: msg.color });
  Ok(())
});

#[derive(Debug, Clone, Message, Serialize)]
pub struct CellUpdated {
  pub x: usize,
  pub y: usize,
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
