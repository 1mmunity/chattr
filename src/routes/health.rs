use warp::Reply;
use crate::{Result, ALLOCATOR};

pub async fn handler() -> Result<impl Reply> {
  Ok(format!("OK - Currently using {} Bytes", ALLOCATOR.allocated()))
}