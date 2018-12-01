pub type Color = u8;

#[derive(Debug)]
pub struct Canvas {
  pub width: usize,
  pub height: usize,
  data: Vec<Color>,
}

impl Canvas {
  pub fn new(width: usize, height: usize, data: Vec<Color>) -> Self {
    assert!(data.len() == width * height);
    Self { width, height, data }
  }

  pub fn get(&self, x: usize, y: usize) -> Color {
    self.assert_in_bounds(x, y);
    self.data[y * self.width + x]
  }

  pub fn set(&mut self, x: usize, y: usize, color: Color) {
    self.assert_in_bounds(x, y);
    self.data[y * self.width + x] = color;
  }

  fn assert_in_bounds(&self, x: usize, y: usize) {
    let w = self.width;
    let h = self.height;

    assert!(x < w, "x out of bounds: the width is {} but the x is {}", w, x);
    assert!(y < h, "y out of bounds: the height is {} but the y is {}", h, y);
  }
}
