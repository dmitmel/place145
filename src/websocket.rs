use actix::prelude::*;
use actix_web::ws;

use State;

pub struct Ws;

impl Actor for Ws {
  type Context = ws::WebsocketContext<Self, State>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    info!("{:?}", msg);
    match msg {
      ws::Message::Text(text) => ctx.text(text),
      ws::Message::Binary(bin) => ctx.binary(bin),
      ws::Message::Ping(msg) => ctx.pong(&msg),
      _ => (),
    }
  }
}
