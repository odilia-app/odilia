use breadx::Error;
use breadx::{prelude::*, display::DisplayConnection, protocol::xproto, Result};
use tokio;
use breadx::protocol::xproto::GrabMode;
use breadx::protocol::xproto::ModMask;
use breadx::protocol::Event;
use breadx::rt_support::tokio_support::connect;
use breadx::protocol::xproto::KeyPressEvent;
use breadx::protocol::xkb::KeySymMap;

#[tokio::main]
async fn main() -> Result<()> {
  // establish a connection to the X11 server
  let mut connection = connect(None).await?;
  // get root window (all windows decend from)
  let root = connection.default_screen().root;
  let _gk = connection.grab_key(
    false, // do NOT report the events normally further down the stack
    root, // the grab window (in the case of root, that's for all windows)
    ModMask::ANY, // no modifiers
    33, // the p key? Into<Grab>
    GrabMode::ASYNC, // pointer mode
    GrabMode::ASYNC // keyboard mode
  ).await;
  loop {
    let event = connection.wait_for_event().await?;
    match event {
      Event::KeyPress(kpe) => {
        println!("KPE recv.");
        println!("{:?}", kpe);
      }
      Event::Expose(_exe) => {
        println!("EXPOSE!");       
      }
      _ => {
        println!("Event recv.");
      }
    }
  }
  Ok(())
}
