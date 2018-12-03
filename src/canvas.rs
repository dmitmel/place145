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

impl Handler<GetCell> for Canvas {
  type Result = <GetCell as Message>::Result;

  fn handle(&mut self, msg: GetCell, _: &mut Self::Context) -> Self::Result {
    if let Some(err) = self.assert_in_bounds(msg.x, msg.y) {
      return Err(err);
    }

    Ok(self.data[msg.y * self.width + msg.x])
  }
}

#[derive(Debug, Deserialize, Message)]
#[rtype("Result<(), String>")]
pub struct UpdateCell {
  pub x: usize,
  pub y: usize,
  pub color: u8,
}

impl Handler<UpdateCell> for Canvas {
  type Result = <UpdateCell as Message>::Result;

  fn handle(&mut self, msg: UpdateCell, _: &mut Self::Context) -> Self::Result {
    if let Some(err) = self.assert_in_bounds(msg.x, msg.y) {
      return Err(err);
    }

    self.data[msg.y * self.width + msg.x] = msg.color;
    Ok(())
  }
}
