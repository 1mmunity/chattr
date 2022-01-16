use serde::{Serialize, Deserialize};
use crate::{Date, DbPool, get_db_con};

const ROOM_TABLE: &str = "rooms.rooms";

pub enum RoomError {
  CannotGetDatabaseConnection(crate::Error),
  CannotCreateRoom(tokio_postgres::Error)
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
  pub fn new(id: i32, name: String, description: Option<String>, owner_id: i32, created_at: Date) -> Self {
    Self {
      id,
      name,
      description,
      owner_id,
      created_at
    }
  }
}

struct RoomManager {
  pub db_pool: DbPool
}

impl RoomManager {
  pub fn new(db_pool: DbPool) -> Self {
    Self {
      db_pool
    }
  }

  pub async fn create_room(
    &mut self,
    name: String,
    description: Option<String>,
    owner_id: i32
    // created_at: Date
  ) -> Result<Room, RoomError> {
    let client = get_db_con(&self.db_pool).await.map_err(RoomError::CannotGetDatabaseConnection)?;
    let rows = match description {
      Some(v) => {
        let query = format!("INSERT INTO {} (name, owner_id, description) VALUES ($1, $2, $3) RETURNING *", ROOM_TABLE);
        client.query(&query, &[&name, &owner_id, &v]).await.map_err(RoomError::CannotCreateRoom)?
      },
      None => {
        let query = format!("INSERT INTO {} (name, owner_id) VALUES ($1, $2) RETURNING *", ROOM_TABLE);
        client.query(&query, &[&name, &owner_id]).await.map_err(RoomError::CannotCreateRoom)?
      }
    };
    
    let row = &rows[0];
    
    Ok(
      Room::new(
        row.get("id"),
        row.get("name"),
        row.get("description"),
        row.get("owner_id"),
        row.get("created_at")
      )
    )
  }
}
