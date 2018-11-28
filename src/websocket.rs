use ws::{CloseCode, Handshake, Message, Sender};

#[derive(Debug)]
pub struct Handler {
  client: Sender,
}

impl Handler {
  pub fn new(client: Sender) -> Self {
    Self { client }
  }
}

impl Handler {
  pub fn client_id(&self) -> u32 {
    self.client.connection_id()
  }
}

impl ws::Handler for Handler {
  fn on_open(&mut self, handshake: Handshake) -> ws::Result<()> {
    let id = self.client_id();
    let address = handshake.remote_addr().unwrap().unwrap();
    info!("connection from [{}] with ID [{}]", address, id);

    Ok(())
  }

  fn on_message(&mut self, msg: Message) -> ws::Result<()> {
    let id = self.client_id();
    info!("message from [{}]: {}", id, msg);

    self.client.send(msg)
  }

  fn on_close(&mut self, code: CloseCode, reason: &str) {
    let id = self.client_id();

    // this is a little shorthand function
    fn s(input: &str) -> String {
      input.to_string()
    }

    let code_id: u16 = code.into();
    let code_desc: String = match code_id {
      1000 => s("Normal Closure"),
      1001 => s("Going Away"),
      1002 => s("Protocol Error"),
      1003 => s("Unsupported Data"),
      1005 => s("No Status Received"),
      1006 => s("Abnormal Closure"),
      1007 => s("Invalid frame payload data"),
      1008 => s("Policy Violation"),
      1009 => s("Message too big"),
      1010 => s("Missing Extension"),
      1011 => s("Internal Error"),
      1012 => s("Service Restart"),
      1013 => s("Try Again Later"),
      1015 => s("Bad Gateway"),
      _ => format!("{}", code_id),
    };

    info!(
      "[{}] has disconnected: {} {}: {}",
      id,
      code_id,
      code_desc,
      if reason.is_empty() { "<no reason>" } else { reason }
    );
  }
}
