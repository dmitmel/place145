use actix_web::*;
use failure::{Fallible, ResultExt};
use log::*;

use crate::config::ServerConfig;
use crate::State;

pub fn start(config: ServerConfig, state: State) -> Fallible<()> {
  let static_files_config = config.static_files;
  let http_server = server::new(move || {
    App::with_state(state.clone())
      .middleware(middleware::Logger::new(r#"%a "%r" %s, %b bytes, %D ms"#))
      .resource("/api/canvas", |r| r.with(routes::api::canvas))
      .resource("/api/connect", |r| r.with(routes::api::connect))
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

mod routes {
  pub mod api {
    use actix_web::actix::*;
    use actix_web::*;
    use futures::Future;

    use crate::canvas::GetCanvas;
    use crate::client::Client;
    use crate::State;

    pub fn canvas(req: HttpRequest<State>) -> FutureResponse<Binary> {
      let canvas_addr = req.state().clone();

      Box::new(
        canvas_addr
          .send(GetCanvas)
          .map_err(|send_error: MailboxError| panic!(send_error))
          .map(Binary::from),
      )
    }

    pub fn connect(req: HttpRequest<State>) -> actix_web::Result<HttpResponse> {
      ws::start(&req, Client::new())
    }
  }
}
