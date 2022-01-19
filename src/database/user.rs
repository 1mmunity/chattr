
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use tokio_postgres::Row;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use validator::Validate;
use crate::{DbCon, Date};

const USER_TABLE: &str = "users";
const SECRET_KEY: &str = dotenv_codegen::dotenv!("SECRET_KEY");

pub enum UserError {
  CannotCreateUser(tokio_postgres::Error),
  UserNotFound,
  CannotGenerateToken(jsonwebtoken::errors::Error),
  CannotDecodeToken(jsonwebtoken::errors::Error),
  InvalidTokenType,
  TokenExpired,
  TokenTypeDoesNotMatch
}

#[derive(Serialize, Deserialize, Validate)]
pub struct User {
  id: i32,

  #[validate(length(min = 3, max = 32))]
  username: String,

  #[validate(email)]
  email: String,

  #[validate(length(min = 3, max = 64))]
  password: String,
  
  token: String,
  created_at: Date
}

impl User {
  pub fn from_row(row: &Row) -> Self {
    Self {
      id: row.get("id"),
      username: row.get("username"),
      email: row.get("email"),
      password: row.get("password"),
      token: row.get("token"),
      created_at: row.get("created_at")
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
  exp: Date,
  iat: Date,
  aud: String,
  id: i32,
  username: String,
  password: String
}
// things to be tokenized

pub enum TokenType {
  AccessToken,
  RefreshToken
}

pub struct UserManager {
  pub db_con: DbCon
}

impl UserManager {
  pub fn new(db_con: DbCon) -> Self {
    Self {
      db_con
    }
  }

  pub async fn create_user(
    &mut self,
    username: String,
    email: String,
    password: String
  ) -> Result<User, UserError> {
    let row = self.db_con.query_one(&format!("INSERT INTO {} (username, email, password, token) VALUES ($1, $2, $3, $4) RETURNING *", USER_TABLE), &[]).await.map_err(UserError::CannotCreateUser)?;
    Ok(User::from_row(&row))
  }

  pub async fn get_from_token(&mut self, token: String) -> Result<User, UserError> {
    let row = self.db_con.query_one(&format!("SELECT * FROM {} WHERE token=$1", USER_TABLE), &[&token]).await.map_err(|_| UserError::UserNotFound)?;
    Ok(User::from_row(&row))
  }

  pub async fn login(&mut self, email: String, password: String) -> Result<User, UserError> {
    let row = self.db_con.query_one(&format!("SELECT * FROM {} WHERE email=$1 AND password=$2", USER_TABLE), &[&email, &password]).await.map_err(|_| UserError::UserNotFound)?;
    Ok(User::from_row(&row))
  }

  pub fn issue_token(
    id: i32,
    username: String,
    password: String,
    token_type: TokenType
  ) -> Result<String, UserError> {
    let (dur, aud) = match token_type {
      TokenType::AccessToken => (Duration::minutes(15), "access"),
      TokenType::RefreshToken => (Duration::hours(24), "refresh")
    };

    encode(
      &Header::default(),
      &UserClaim {
        exp: Utc::now() + dur,
        iat: Utc::now(),
        aud: aud.into(),
        id,
        username,
        password
      },
      &EncodingKey::from_secret(SECRET_KEY.as_ref())
    ).map_err(UserError::CannotGenerateToken)
  }

  pub fn reissue_token(old_token: String, token_type: TokenType) -> Result<String, UserError> {
    let decoded: UserClaim = decode(
      &old_token,
      &DecodingKey::from_secret(SECRET_KEY.as_ref()),
      &Validation::default()
    )
    .map_err(UserError::CannotDecodeToken)?
    .claims;

    // this way of comparing enums a bit inefficient can sum1 do any diff?
    let token_type_str: &str = match token_type {
      TokenType::AccessToken => "access",
      TokenType::RefreshToken => "refresh"
    };

    // checks if decoded token is not access token when refresh token is expected and vice versa
    if token_type_str != decoded.aud.as_str() {
      return Err(UserError::TokenTypeDoesNotMatch);
    }

    // check if token has expired.
    if decoded.exp < Utc::now() {
      return Err(UserError::TokenExpired);
    }

    Ok(
      Self::issue_token(
        decoded.id,
        decoded.username,
        decoded.password,
        token_type
      )?
    )
  }
}
