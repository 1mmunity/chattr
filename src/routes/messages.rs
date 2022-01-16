use warp::Reply;
use crate::{DbPool, Result, database::message::Message};

pub async fn post_to_room(room_id: i32, body: Message, db_pool: DbPool) -> Result<impl Reply> {
  Ok("m")
}