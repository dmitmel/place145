use actix_web::*;
use futures::Future;

use canvas::{GetCell, UpdateCell};
use State;

pub fn get_cell(
  (request, query): (HttpRequest<State>, Query<GetCell>),
) -> FutureResponse<String> {
  let canvas_addr = request.state();
  Box::new(canvas_addr.send(query.into_inner()).then(|result| match result {
    Ok(Ok(color)) => Ok(color.to_string()),
    Ok(Err(error)) => Ok(error.to_string()),
    Err(error) => panic!(error),
  }))
}

pub fn update_cell(
  (request, query): (HttpRequest<State>, Query<UpdateCell>),
) -> FutureResponse<String> {
  let canvas_addr = request.state();
  Box::new(canvas_addr.send(query.into_inner()).then(|result| match result {
    Ok(Ok(_)) => Ok("ok".to_string()),
    Ok(Err(error)) => Ok(error.to_string()),
    Err(error) => panic!(error),
  }))
}
