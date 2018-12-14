use log::*;
use serde_derive::*;

use actix::prelude::*;
use actix_web::ws;
use futures::Future;

use crate::canvas::*;
use crate::State;

#[derive(Debug)]
pub struct Client;

type Context = ws::WebsocketContext<Client, State>;

impl Actor for Client {
  type Context = Context;

  fn started(&mut self, ctx: &mut Self::Context) {
    let addr = ctx.address();
    self.send_to_canvas(ListenerConnected { addr }, ctx, |_, _, _| {
      actix::fut::ok(())
    });
  }

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    let addr = ctx.address();
    self.send_to_canvas(ListenerDisconnected { addr }, ctx, |_, _, _| {
      actix::fut::ok(())
    });
    Running::Stop
  }
}

impl Client {
  pub fn send_to_canvas<M: 'static, I: 'static, F: 'static, B: 'static>(
    &mut self,
    msg: M,
    ctx: &mut Context,
    then: F,
  ) where
    M: Message<Result = I> + Send,
    I: Send,
    Canvas: Handler<M>,
    F: FnOnce(Result<I, MailboxError>, &mut Client, &mut Context) -> B,
    B: ActorFuture<Item = (), Error = (), Actor = Client> + Sized,
  {
    {
      let canvas_addr = ctx.state();
      canvas_addr.send(msg)
    }
    .map_err(|send_error: MailboxError| panic!(send_error))
    .into_actor(self)
    .then(then)
    .wait(ctx)
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Client {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Binary(binary) => self.handle_packet(binary.as_ref(), ctx),
      ws::Message::Ping(msg) => ctx.pong(&msg),
      _ => info!("{:?}", msg),
    }
  }
}

#[derive(Debug, Deserialize)]
enum RequestPacket {
  GetCell { x: Coord, y: Coord },
  SetCell { x: Coord, y: Coord, color: Color },
}

#[derive(Debug, Serialize)]
enum ResponsePacket {
  Error { message: String },
  CellData { x: Coord, y: Coord, color: Color },
  CellUpdated { x: Coord, y: Coord, color: Color },
}

fn send_packet(packet: ResponsePacket, ctx: &mut Context) {
  info!("sending packet {:?}", packet);
  ctx.binary(bincode::config().big_endian().serialize(&packet).unwrap());
}

fn send_error(message: &str, ctx: &mut Context) {
  send_packet(ResponsePacket::Error { message: message.to_string() }, ctx)
}

impl Client {
  fn handle_packet(&mut self, bytes: &[u8], ctx: &mut Context) {
    let packet: RequestPacket =
      match bincode::config().big_endian().deserialize(&bytes[..]) {
        Ok(packet) => packet,
        Err(error) => {
          send_error(&error.to_string(), ctx);
          return;
        }
      };

    info!("received packet {:?}", packet);

    use self::RequestPacket::*;
    match packet {
      GetCell { x, y } => self.get_cell(x, y, ctx),
      SetCell { x, y, color } => self.set_cell(x, y, color, ctx),
    }
  }

  fn get_cell(&mut self, x: Coord, y: Coord, ctx: &mut Context) {
    self.send_to_canvas(GetCell { x, y }, ctx, move |result, _, ctx| {
      match result.unwrap() {
        Ok(color) => send_packet(ResponsePacket::CellData { x, y, color }, ctx),
        Err(error) => send_error(&error, ctx),
      }
      actix::fut::ok(())
    });
  }

  fn set_cell(&mut self, x: Coord, y: Coord, color: Color, ctx: &mut Context) {
    self.send_to_canvas(UpdateCell { x, y, color }, ctx, |result, _, ctx| {
      if let Err(error) = result.unwrap() {
        send_error(&error, ctx)
      }
      actix::fut::ok(())
    })
  }
}

actix_handler!(CellUpdated, Client, |_, msg, ctx| {
  let CellUpdated { x, y, color } = msg;
  send_packet(ResponsePacket::CellUpdated { x, y, color }, ctx);
});
