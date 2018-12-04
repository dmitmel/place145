use actix::prelude::*;
use actix_web::ws;

use serde_json;

use canvas::*;
use State;

#[derive(Debug)]
pub struct Ws;

impl Actor for Ws {
  type Context = ws::WebsocketContext<Self, State>;

  fn started(&mut self, ctx: &mut Self::Context) {
    let addr = ctx.address();
    self.send_to_canvas(ListenerConnected { addr }, ctx);
  }

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    let addr = ctx.address();
    self.send_to_canvas(ListenerDisconnected { addr }, ctx);
    Running::Stop
  }
}

impl Ws {
  pub fn send_to_canvas<M: 'static>(
    &mut self,
    msg: M,
    ctx: &mut <Self as Actor>::Context,
  ) where
    M: Message<Result = ()> + Send,
    Canvas: Handler<M>,
  {
    {
      let canvas_addr = ctx.state();
      canvas_addr.send(msg)
    }
    .into_actor(self)
    .then(|_, _, _| actix::fut::ok(()))
    .wait(ctx)
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    info!("{:?}", msg);
    if let ws::Message::Ping(msg) = msg {
      ctx.pong(&msg)
    }
  }
}

actix_handler!(CellUpdated, Ws, |_, msg, ctx| {
  ctx.text(serde_json::to_string(&msg).unwrap());
});
