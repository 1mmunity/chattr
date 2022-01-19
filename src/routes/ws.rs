use crate::{database, ws_messages};
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc
};
use warp::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde::{Serialize, Deserialize};

use tokio::time::{timeout, Duration};

// this is more of a handler than a route itself.

type Socket = mpsc::UnboundedSender<Message>;

pub struct WsClient {
  socket: Socket,
  user: Option<database::user::User>
}

impl WsClient {
  fn default(socket: Socket) -> Self {
    Self {
      socket,
      user: None
    }
  }
}

pub type WsClients = Arc<RwLock<HashMap<usize, WsClient>>>;

const USER_NOT_LOGIN_TIMEOUT: u64 = 20000;
const LATENCY_COMPENSATION: u64 = 2500;

pub mod WsCodes {
  const DISCONNECT: u8 = 0; // client disconnect due to error or other
  const EVENT: u8 = 1; // will only send after the user has logged in through the websocket
  const USER_LOGIN: u8 = 11;
}

pub mod WsEvents {
  const LOGIN: u8 = 0; // when the user has logged in using the ws
  const ROOM_MESSAGE: u8 = 1; // when the user got a room message
  const ROOM_JOIN: u8 = 2;
  const ROOM_LEAVE: u8 = 3;
}

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage<T> {
  a: u8, // WsCodes
  e: Option<u8>, // event name if code is 1 (nullable)
  i: Option<String>, // session id (unavailable right now)
  d: T // data
}

pub async fn connect_user(
  ws: WebSocket,
  clients: WsClients,
  next_id: &AtomicUsize 
) {
  let id = next_id.fetch_add(1, Ordering::Relaxed);

  let (mut client_tx, mut client_rx) = ws.split();
  let (tx, rx) = mpsc::unbounded_channel();
  let mut rx = UnboundedReceiverStream::new(rx);

  let handle = tokio::task::spawn(async move {
    while let Some(message) = rx.next().await {
      client_tx
        .send(message)
        .unwrap_or_else(|e| {
            eprintln!("websocket send error: {}", e);
        })
        .await;
    }
  });
  
  println!("User connected: {}", id);
  clients.write().await.insert(id, WsClient::default(tx));

  if let Ok(v) = timeout(Duration::from_millis(USER_NOT_LOGIN_TIMEOUT + LATENCY_COMPENSATION), client_rx.next()).await {
    if let Some(result) = v {
      let msg = match result {
        Ok(msg) => msg,
        Err(e) => {
          eprintln!("websocket error(uid={}): {}", id, e);
          return;
        }
      };

      let v: WsMessage<ws_messages::UserLogin> = serde_json::from_str(&msg.clone().to_str().unwrap()).unwrap();
      println!("{:?}", v);

      // ... fetches the user and bind it to WsClient...
    };
  } else {
    println!("Disconnected...");
    handle.abort(); // aborts the connection
    return
  };

  while let Some(result) = client_rx.next().await {
    let msg = match result {
      Ok(msg) => msg,
      Err(e) => {
        eprintln!("websocket error(uid={}): {}", id, e);
        break;
      }
    };

    match handle_message(id, msg, &clients).await {
      Ok(v) => v,
      Err(_) => {
        break;
      }
    };
  }

  handle_disconnect(id, &clients).await;
}

async fn handle_disconnect(id: usize, clients: &WsClients) {
  println!("User disconnected: {}", id);
  clients.write().await.remove(&id);
}

async fn handle_message(id: usize, message: Message, clients: &WsClients) -> Result<(), ()> {
  let msg = message.to_str().unwrap();
  for (&uid, client) in clients.read().await.iter() {
    // if id != uid {
      if let Err(_dc) = client.socket.send(Message::text(msg.clone())) {

      };
    // }
  };

  Ok(())
}

async fn handle_login() -> Result<(), ()> {
  Ok(())
}