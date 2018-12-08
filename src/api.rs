use actix::prelude::*;
use actix_web::*;
use futures::Future;

use canvas::GetCanvas;
use State;

pub fn canvas(request: HttpRequest<State>) -> FutureResponse<Binary> {
  let canvas_addr = request.state().clone();

  Box::new(
    canvas_addr
      .send(GetCanvas)
      .map_err(|send_error: MailboxError| panic!(send_error))
      .map(Binary::from),
  )
}
