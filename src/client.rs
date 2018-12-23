use std::time::{Duration, Instant};

use log::*;
use serde_derive::*;

use actix::prelude::*;
use actix_web::ws;
use futures::Future;

use crate::canvas::*;
use crate::State;

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Client {
  remote_addr: String,
  last_pong_time: Instant,
}

#[allow(clippy::new_without_default)]
impl Client {
  pub fn new() -> Self {
    Self {
      remote_addr: "<unknown>".to_string(),
      last_pong_time: Instant::now(),
    }
  }
}

type Context = ws::WebsocketContext<Client, State>;

impl Actor for Client {
  type Context = Context;

  fn started(&mut self, ctx: &mut Context) {
    if let Some(addr) = ctx.request().connection_info().remote() {
      self.remote_addr = addr.to_string();
    }

    info!("{} connected", self.remote_addr);

    let addr = ctx.address();
    self.send_to_canvas(ListenerConnected { addr }, ctx, |_, _, _| {
      actix::fut::ok(())
    });

    self.start_sending_pings(ctx);
  }

  fn stopping(&mut self, ctx: &mut Context) -> Running {
    let addr = ctx.address();
    self.send_to_canvas(ListenerDisconnected { addr }, ctx, |_, _, _| {
      actix::fut::ok(())
    });
    Running::Stop
  }
}

impl Client {
  fn start_sending_pings(&self, ctx: &mut Context) {
    ctx.run_interval(PING_INTERVAL, |self_, ctx| {
      if Instant::now().duration_since(self_.last_pong_time) > PING_TIMEOUT {
        info!("'ping' to {} timed out", self_.remote_addr);
        ctx.close(Some(ws::CloseReason {
          code: ws::CloseCode::Abnormal,
          description: Some("ping timed out".to_string()),
        }));
      } else {
        info!("sending 'ping' to {}", self_.remote_addr);
        ctx.ping("");
      }
    });
  }

  fn send_to_canvas<M: 'static, I: 'static, F: 'static, B: 'static>(
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
  fn handle(&mut self, msg: ws::Message, ctx: &mut Context) {
    match msg {
      ws::Message::Binary(binary) => self.handle_packet(binary.as_ref(), ctx),

      ws::Message::Ping(msg) => ctx.pong(&msg),
      ws::Message::Pong(_) => {
        info!("received 'pong' from {}", self.remote_addr);
        self.last_pong_time = Instant::now();
      }

      ws::Message::Close(reason) => {
        let addr = &self.remote_addr;
        if let Some(ws::CloseReason { code, description }) = reason {
          let code: u16 = code.into();
          if let Some(description) = description {
            info!("{} disconnected with code {}: {}", addr, code, description);
          } else {
            info!("{} disconnected with code {}", addr, code);
          }
        } else {
          info!("{} disconnected", addr);
        }
      }

      _ => {}
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

impl Client {
  fn send_packet(&self, packet: ResponsePacket, ctx: &mut Context) {
    info!("sending packet {:?} to {}", packet, self.remote_addr);
    ctx.binary(bincode::config().big_endian().serialize(&packet).unwrap());
  }

  fn send_error(&self, msg: &str, ctx: &mut Context) {
    self.send_packet(ResponsePacket::Error { message: msg.to_string() }, ctx)
  }

  fn handle_packet(&mut self, bytes: &[u8], ctx: &mut Context) {
    let packet: RequestPacket =
      match bincode::config().big_endian().deserialize(&bytes[..]) {
        Ok(packet) => packet,
        Err(error) => {
          self.send_error(&error.to_string(), ctx);
          return;
        }
      };

    info!("received packet {:?} from {}", packet, self.remote_addr);

    use self::RequestPacket::*;
    match packet {
      GetCell { x, y } => self.get_cell(x, y, ctx),
      SetCell { x, y, color } => self.set_cell(x, y, color, ctx),
    }
  }

  fn get_cell(&mut self, x: Coord, y: Coord, ctx: &mut Context) {
    self.send_to_canvas(GetCell { x, y }, ctx, move |res, self_, ctx| {
      match res.unwrap() {
        Ok(color) => {
          self_.send_packet(ResponsePacket::CellData { x, y, color }, ctx)
        }
        Err(error) => self_.send_error(&error, ctx),
      }
      actix::fut::ok(())
    });
  }

  fn set_cell(&mut self, x: Coord, y: Coord, color: Color, ctx: &mut Context) {
    self.send_to_canvas(UpdateCell { x, y, color }, ctx, |res, self_, ctx| {
      if let Err(error) = res.unwrap() {
        self_.send_error(&error, ctx)
      }
      actix::fut::ok(())
    })
  }
}

actix_handler!(CellUpdated, Client, |self_, msg, ctx| {
  let CellUpdated { x, y, color } = msg;
  self_.send_packet(ResponsePacket::CellUpdated { x, y, color }, ctx);
});
