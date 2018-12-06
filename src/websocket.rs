use actix::prelude::*;
use actix_web::ws;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use futures::Future;

use canvas::*;
use State;

#[derive(Debug)]
pub struct Ws;

type WsContext = ws::WebsocketContext<Ws, State>;

impl Actor for Ws {
  type Context = WsContext;

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

impl Ws {
  pub fn send_to_canvas<M: 'static, I: 'static, F: 'static, B: 'static>(
    &mut self,
    msg: M,
    ctx: &mut WsContext,
    then: F,
  ) where
    M: Message<Result = I> + Send,
    I: Send,
    Canvas: Handler<M>,
    F: FnOnce(Result<I, MailboxError>, &mut Ws, &mut WsContext) -> B,
    B: ActorFuture<Item = (), Error = (), Actor = Ws> + Sized,
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

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Binary(binary) => self.handle_packet(binary.as_ref(), ctx),
      ws::Message::Ping(msg) => ctx.pong(&msg),
      _ => info!("{:?}", msg),
    }
  }
}

enum PacketType {
  Error,
  GetCell,
  CellData,
  SetCell,
  CellUpdated,
}

fn send_packet(type_: PacketType, payload: &[u8], ctx: &mut WsContext) {
  let mut packet: Vec<u8> = vec![type_ as u8];
  packet.extend(payload);
  ctx.binary(packet);
}

impl Ws {
  #[allow(clippy::string_lit_as_bytes)]
  fn handle_packet(&mut self, mut bytes: &[u8], ctx: &mut WsContext) {
    #[rustfmt::skip]
    macro_rules! read {
      (u8, $msg:expr) =>  { read!(@unwrap bytes.read_u8(), $msg) };
      (u16, $msg:expr) => { read!(@unwrap bytes.read_u16::<NetworkEndian>(), $msg) };

      (@unwrap $result:expr, $msg:expr) => {
        match $result {
          Ok(value) => value,
          Err(_) => {
            send_packet(PacketType::Error, $msg, ctx);
            return;
          },
        }
      };
    }

    let msg_type = read!(u8, b"expected packet type");
    let x = read!(u16, b"expected cell x");
    let y = read!(u16, b"expected cell y");
    if msg_type == PacketType::GetCell as u8 {
      self.handle_get_cell(x, y, ctx);
    } else if msg_type == PacketType::SetCell as u8 {
      let color = read!(u8, b"expected color");
      self.handle_set_cell(x, y, color, ctx);
    } else {
      send_packet(PacketType::Error, b"unknown packet type", ctx)
    }
  }

  fn handle_get_cell(&mut self, x: Coord, y: Coord, ctx: &mut WsContext) {
    self.send_to_canvas(GetCell { x, y }, ctx, move |result, _, ctx| {
      match result.unwrap() {
        Ok(color) => {
          let mut response: Vec<u8> = vec![];
          response.write_u16::<NetworkEndian>(x).unwrap();
          response.write_u16::<NetworkEndian>(y).unwrap();
          response.write_u8(color).unwrap();
          send_packet(PacketType::CellData, &response, ctx);
        }
        Err(error) => {
          send_packet(PacketType::Error, error.as_bytes(), ctx);
        }
      }
      actix::fut::ok(())
    });
  }

  fn handle_set_cell(
    &mut self,
    x: Coord,
    y: Coord,
    color: Color,
    ctx: &mut WsContext,
  ) {
    self.send_to_canvas(UpdateCell { x, y, color }, ctx, |result, _, ctx| {
      if let Err(error) = result.unwrap() {
        send_packet(PacketType::Error, error.as_bytes(), ctx);
      }
      actix::fut::ok(())
    })
  }
}

actix_handler!(CellUpdated, Ws, |_, msg, ctx| {
  let mut response: Vec<u8> = vec![];
  response.write_u16::<NetworkEndian>(msg.x).unwrap();
  response.write_u16::<NetworkEndian>(msg.y).unwrap();
  response.write_u8(msg.color).unwrap();
  send_packet(PacketType::CellUpdated, &response, ctx);
});
