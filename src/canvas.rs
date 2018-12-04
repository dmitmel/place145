use actix::prelude::*;

pub type Color = u8;

#[derive(Debug)]
pub struct Canvas {
  pub width: usize,
  pub height: usize,
  data: Vec<Color>,
}

impl Actor for Canvas {
  type Context = Context<Self>;
}

impl Canvas {
  pub fn new(width: usize, height: usize) -> Self {
    Self { width, height, data: vec![0; width * height] }
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
  Ok(())
});
