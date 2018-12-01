use actix_web::{HttpRequest, Query};

use State;

#[derive(Debug, Deserialize)]
pub struct GetCell {
  pub x: usize,
  pub y: usize,
}

pub fn get_cell(
  (request, query): (HttpRequest<State>, Query<GetCell>),
) -> String {
  let GetCell { x, y } = *query;

  let state = request.state().lock().unwrap();
  format!("{}", state.get(x, y))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCell {
  pub x: usize,
  pub y: usize,
  pub color: u8,
}

pub fn update_cell(
  (request, query): (HttpRequest<State>, Query<UpdateCell>),
) -> String {
  let UpdateCell { x, y, color } = *query;

  let mut state = request.state().lock().unwrap();
  state.set(x, y, color);
  format!("{}", state.get(x, y))
}
