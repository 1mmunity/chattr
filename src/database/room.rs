use serde::{Serialize, Deserialize};
use tokio_postgres::Row;
use crate::{Date, DbCon};

const ROOM_TABLE: &str = "rooms.rooms";

pub enum RoomError {
  CannotCreateRoom(tokio_postgres::Error),
  RoomNotFound
}

#[derive(Serialize, Deserialize)]
struct Room {
  id: i32,
  name: String,
  description: Option<String>,
  owner_id: i32,
  created_at: Date
}

impl Room {
  pub fn from_row(row: &Row) -> Self {
    Self {
      id: row.get("id"),
      name: row.get("name"),
      description: row.get("description"),
      owner_id: row.get("owner_id"),
      created_at: row.get("created_at")
    }
  }
}

struct RoomManager {
  pub db_con: DbCon
}

impl RoomManager {
  pub fn new(db_con: DbCon) -> Self {
    Self {
      db_con
    }
  }

  pub async fn create_room(
    &mut self,
    name: String,
    description: Option<String>,
    owner_id: i32
    // created_at: Date
  ) -> Result<Room, RoomError> {
    let row = match description {
      Some(v) => {
        self.db_con.query_one(&format!("INSERT INTO {} (name, owner_id, description) VALUES ($1, $2, $3) RETURNING *", ROOM_TABLE), &[&name, &owner_id, &v]).await.map_err(RoomError::CannotCreateRoom)?
      },
      None => {
        self.db_con.query_one(&format!("INSERT INTO {} (name, owner_id) VALUES ($1, $2) RETURNING *", ROOM_TABLE), &[&name, &owner_id]).await.map_err(RoomError::CannotCreateRoom)?
      }
    };
    
    Ok(Room::from_row(&row))
  }

  pub async fn delete_room(
    &mut self,
    id: i32
  ) -> Result<(), RoomError> {
    self.db_con.query_one(&format!("DELETE {} FROM users WHERE id=$1", ROOM_TABLE), &[&id]).await.map_err(|_| RoomError::RoomNotFound)?;
    Ok(())
  }

  pub async fn update_room_name(
    &mut self,
    room_id: i32,
    new_name: String
  ) -> Result<(), RoomError> {
    self.db_con.query_one(&format!("UPDATE {} SET name = $1 WHERE id=$2", ROOM_TABLE), &[&new_name, &room_id]).await.map_err(|_| RoomError::RoomNotFound)?;
    Ok(())
  }

  pub async fn update_room_description(
    &mut self,
    room_id: i32,
    new_description: String
  ) -> Result<(), RoomError> {
    self.db_con.query_one(&format!("UPDATE {} SET description = $1 WHERE id=$2", ROOM_TABLE), &[&new_description, &room_id]).await.map_err(|_| RoomError::RoomNotFound)?;
    Ok(())
  }
}
