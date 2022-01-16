#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate dotenv_codegen;

mod database;
mod routes;

use mobc::Pool;
use mobc_postgres::PgConnectionManager;
use tokio_postgres::{Config, NoTls};
use warp::{Filter, Rejection};
use std::convert::Infallible;
use std::time::Duration;
use std::str::FromStr;
use futures_util::{FutureExt, StreamExt};
use cap::Cap;

pub type Date = chrono::DateTime<chrono::Utc>;
pub type DbManager = mobc_postgres::PgConnectionManager<tokio_postgres::NoTls>;
pub type DbPool = mobc::Pool<DbManager>;
pub type DbCon = mobc::Connection<DbManager>;
pub type Result<T> = std::result::Result<T, Rejection>;

pub enum Error {
    CannotGetDatabaseConnection(mobc::Error<tokio_postgres::Error>)
}

#[global_allocator]
static ALLOCATOR: Cap<std::alloc::System> = Cap::new(std::alloc::System, usize::MAX);

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
    .ok()
    .map(|val| val.parse::<u16>().unwrap())
    .unwrap_or(3001);

    // let db_pool = init_pool().await;
    // println!("CONNECTED TO DATABASE");

    let routes = warp::any()
    .and(warp::path::end())
    .and_then(routes::health::handler)
    .or(
        warp::path::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|client| {
                let (tx, rx) = client.split();
                rx.forward(tx).map(|res| {
                    if let Err(e) = res {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        })
    );
    // .or(
    //     warp::path!("rooms" / i32 / "messages")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and(with_db(db_pool.clone()))
    //     .and_then(routes::messages::post_to_room)
    // );

    warp::serve(routes)
    .tls()
    .key_path("./tls/key.pem")
    .cert_path("./tls/cert.pem")
    .run(([127, 0, 0, 1], port))
    .await;
}

// some database functions
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

fn with_db(db_pool: DbPool) -> impl Filter<Extract = (DbPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}
