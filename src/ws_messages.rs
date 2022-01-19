use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct UserLogin {
  token: String,
  // session_id: Option<String> // unsupported atm
}