use actix::prelude::*;
use actix_web::http::StatusCode;
use actix_web::*;
use futures::{Future, IntoFuture};

use canvas::Canvas;
use State;

pub fn canvas_message<M: 'static, I: 'static, R: 'static>(
  (request, query): (HttpRequest<State>, Result<Query<M>>),
  build_response: R,
) -> FutureResponse<HttpResponse>
where
  M: Message<Result = Result<I, String>> + Send,
  I: Send,
  Canvas: Handler<M>,
  R: FnOnce(I) -> String,
{
  let canvas_addr = request.state().clone();

  Box::new(
    query
      .into_future()
      .map_err(|error: actix_web::Error| error.to_string())
      .and_then(move |query: Query<M>| {
        canvas_addr
          .send(query.into_inner())
          .map_err(|send_error: MailboxError| panic!(send_error))
          .and_then(|msg_response| msg_response)
      })
      .then(move |result: Result<I, String>| -> Result<HttpResponse> {
        let (status_code, body) = match result {
          Ok(value) => (StatusCode::OK, build_response(value)),
          Err(error) => (StatusCode::BAD_REQUEST, error),
        };

        let response = request
          .build_response(status_code)
          .content_type("text/plain; charset=utf-8")
          .body(body);
        Ok(response)
      }),
  )
}
