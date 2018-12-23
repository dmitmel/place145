use actix_web::*;
use failure::{Fallible, ResultExt};
use futures::Future;
use log::*;

use actix_web::{middleware, server, ws, App};

use crate::canvas::GetCanvas;
use crate::client::Client;
use crate::config::ServerConfig;
use crate::State;

pub fn start(config: ServerConfig, state: State) -> Fallible<()> {
  let static_files_config = config.static_files;
  let http_server = server::new(move || {
    App::with_state(state.clone())
      .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
      .resource("/api/canvas", |r| r.with(api_canvas))
      .resource("/api/connect", |r| r.with(api_stream))
      .handler(
        &static_files_config.base_url,
        actix_web::fs::StaticFiles::new(&static_files_config.path)
          .unwrap()
          .index_file(static_files_config.index_file.clone()),
      )
  });

  info!("starting HTTP server");
  http_server
    .bind(config.address)
    .context("couldn't bind server socket")?
    .start();

  Ok(())
}

fn api_canvas(request: HttpRequest<State>) -> FutureResponse<Binary> {
  let canvas_addr = request.state().clone();

  Box::new(
    canvas_addr
      .send(GetCanvas)
      .map_err(|send_error| panic!(send_error))
      .map(Binary::from),
  )
}

fn api_stream(request: HttpRequest<State>) -> actix_web::Result<HttpResponse> {
  ws::start(&request, Client::new())
}
