#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate rocket;
extern crate ws;

mod routes;
mod websocket;

fn main() {
  env_logger::init();

  let server = rocket::ignite().mount("/", routes![routes::index]);
  start_websocket(server.config());
  server.launch();
}

fn start_websocket(config: &rocket::Config) {
  let hostname = config.address.clone();
  let port = config.get_int("websocket_port").unwrap() as u16;

  use std::thread;
  thread::Builder::new()
    .name("websocket".to_string())
    .spawn(move || {
      let address: (&str, u16) = (&hostname, port);
      ws::listen(address, websocket::Handler::new).unwrap()
    })
    .unwrap();
}
