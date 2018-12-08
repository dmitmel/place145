macro_rules! actix_handler {
  ($msg:ty, $act:path, $cb:expr) => {
    impl ::actix::Handler<$msg> for $act {
      type Result = ::actix::MessageResult<$msg>;

      fn handle(&mut self, msg: $msg, ctx: &mut Self::Context) -> Self::Result {
        let cb: fn(
          &mut $act,
          $msg,
          &mut Self::Context,
        ) -> <$msg as ::actix::Message>::Result = $cb;
        ::actix::MessageResult(cb(self, msg, ctx))
      }
    }
  };
}
