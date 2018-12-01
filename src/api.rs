use actix_web::{HttpRequest, Query};

use State;

#[derive(Deserialize)]
pub struct GetCell {
  x: usize,
  y: usize,
}

pub fn get_cell(
  (request, data): (HttpRequest<State>, Query<GetCell>),
) -> String {
  let state = request.state().lock().unwrap();
  format!("{}", state.get(data.x, data.y))
}

#[derive(Deserialize)]
pub struct UpdateCell {
  x: usize,
  y: usize,
  color: u8,
}

pub fn update_cell(
  (request, data): (HttpRequest<State>, Query<UpdateCell>),
) -> String {
  let mut state = request.state().lock().unwrap();
  state.set(data.x, data.y, data.color);
  format!("{}", state.get(data.x, data.y))
}
