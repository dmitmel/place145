use actix_web::*;
use futures::Future;

use crate::canvas::GetCanvas;
use crate::State;

pub fn canvas(request: HttpRequest<State>) -> FutureResponse<Binary> {
  let canvas_addr = request.state().clone();

  Box::new(
    canvas_addr
      .send(GetCanvas)
      .map_err(|send_error| panic!(send_error))
      .map(Binary::from),
  )
}
