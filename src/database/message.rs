use serde::{Serialize, Deserialize};
use crate::{Date, DbCon};

const MESSAGE_TABLE: &str = "rooms.messages";
pub enum MessageError {
  CannotPostMessage(tokio_postgres::Error),
  CannotDeleteMessage(tokio_postgres::Error),
  MessageNotFound,
}

#[derive(Serialize, Deserialize)]
pub struct MessageRequest {
  author_id: i32,
  content: String,
  created_at: Date
}

#[derive(Deserialize)]
pub struct Message {
  id: i64,
  author_id: i32,
  content: String,
  created_at: Date // for javascript Date object.
}

impl Message {
  pub fn new(id: i64, author_id: i32, content: String, created_at: Date) -> Self {
    Self {
      id,
      author_id,
      content,
      created_at
    }
  }
}

struct MessageManager {
  pub db_con: DbCon
}

impl MessageManager {
  pub fn new(db_con: DbCon) -> Self {
    Self {
      db_con
    }
  }

  pub async fn create_message(
    &mut self,
    user_id: i32,
    content: String
    // created_at: Date
  ) -> Result<Message, MessageError> {
    // let created_at_or_now = created_at.unwrap_or(chrono::prelude::Utc::now());
    let query = format!("INSERT INTO {} (user_id, content) VALUES ($1, $2, $3) RETURNING *", MESSAGE_TABLE);
    let rows = self.db_con.query(&query, &[&user_id, &content]).await.map_err(MessageError::CannotPostMessage)?;
    let row = &rows[0];
    Ok(
      Message::new(
        row.get("id"),
        row.get("author_id"),
        row.get("content"),
        row.get("created_at")
      )
    )
  }

  pub async fn delete_message(&mut self, message: &Message) -> Result<(), MessageError> {
    let query = format!("DELETE FROM {} WHERE id=$1 RETURNING *", MESSAGE_TABLE);
    let rows = self.db_con.query(&query, &[&message.id]).await.map_err(MessageError::CannotDeleteMessage)?;
    if let Some(_) = &rows.get(0) {
      Ok(())
    } else {
      Err(MessageError::MessageNotFound)
    }
  }
}