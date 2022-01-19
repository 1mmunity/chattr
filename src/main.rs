#[macro_use]
extern crate dotenv_codegen;

mod database;
mod routes;
mod ws_messages;

use routes::ws;

use mobc::Pool;
use cap::Cap;
use mobc_postgres::PgConnectionManager;
use futures_util::{FutureExt, StreamExt};
use std::sync::atomic::AtomicUsize;
use tokio_postgres::{Config, NoTls};
use warp::{Filter, Rejection};
use std::time::Duration;
use std::str::FromStr;

pub type Date = chrono::DateTime<chrono::Utc>;
pub type DbManager = mobc_postgres::PgConnectionManager<tokio_postgres::NoTls>;
pub type DbPool = mobc::Pool<DbManager>;
pub type DbCon = mobc::Connection<DbManager>;
pub type Result<T> = std::result::Result<T, Rejection>;

pub enum Error {
    CannotGetDatabaseConnection(mobc::Error<tokio_postgres::Error>)
}

pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1); 

#[global_allocator]
static ALLOCATOR: Cap<std::alloc::System> = Cap::new(std::alloc::System, usize::MAX);

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
    .unwrap_or("3000".into())
    .parse()
    .expect("PORT must be a number.");

    let users = ws::WsClients::default();

    let db_pool = init_pool().await;
    println!("CONNECTED TO DATABASE");

    let with_db = warp::any().map(move || db_pool.clone());
    let with_clients = warp::any().map(move || users.clone());

    let rts = warp::any()
    .and(warp::path::end())
    .and_then(routes::health::handler)
    .or(
        warp::path::path("ws")
        .and(warp::ws())
        .and(with_clients)
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| routes::ws::connect_user(socket, clients, &NEXT_USER_ID))
        })
    );
    // .or(
    //     warp::path!("rooms" / i32 / "messages")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and(with_db(db_pool.clone()))
    //     .and_then(routes::messages::post_to_room)
    // );

    println!("Starting server on PORT {}", port);

    warp::serve(rts)
    .bind(([0, 0, 0, 0], port))
    .await;
}

// database functions
const DB_POOL_MAX_OPEN: u64 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

pub async fn init_pool() -> DbPool {
    let config = Config::from_str(&dotenv!("DB_URL")).unwrap();
    let manager = PgConnectionManager::new(config, NoTls);
    Pool::builder()
    .max_open(DB_POOL_MAX_OPEN)
    .max_idle(DB_POOL_MAX_IDLE)
    .get_timeout(Some(Duration::new(DB_POOL_TIMEOUT_SECONDS, 0)))
    .build(manager)
}

pub async fn get_db_con(db_pool: &DbPool) -> std::result::Result<DbCon, Error> {
    Ok(db_pool.get().await.map_err(Error::CannotGetDatabaseConnection)?)
}
