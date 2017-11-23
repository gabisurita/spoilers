use std::ops::Deref;

use diesel::{Queryable, Insertable};
use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager as PgConnectionManager;
use r2d2_redis::RedisConnectionManager;
use rocket::{State, Request, Outcome};
use rocket::request::{self, FromRequest};
use rocket::http::Status;
use serde::{Serialize, Deserialize};

use super::DATABASE_URL;



type DatabaseConnectionPool = r2d2::Pool<PgConnectionManager<PgConnection>>;
type CacheConnectionPool = r2d2::Pool<RedisConnectionManager>;


type DatabaseConnection = r2d2::PooledConnection<PgConnectionManager<PgConnection>>;
type CacheConnection = r2d2::PooledConnection<RedisConnectionManager>;


#[derive(Debug,Serialize,Deserialize)]
pub struct StorageBackendError {}

#[derive(Debug,Serialize,Deserialize)]
pub struct CacheBackendError {}

#[derive(Debug,Serialize,Deserialize)]
pub struct QueueBackendError {}


pub trait StorageBackend<Form,Model,Filters,Connection>{
    fn create<'a>(form: Form, conn: &'a Connection)
        -> Result<Model,StorageBackendError>;

    fn list<'a>(filters: Filters, conn: &'a Connection)
        -> Result<Vec<Model>,StorageBackendError>;
}


pub trait QueueBackend {
    fn enqueue<T>(queue: &str, obj: T) -> Result<u32, QueueBackendError> where T: Serialize;
    fn process<'a,T>(queue: &str) -> Result<Vec<T>, QueueBackendError> where T: Deserialize<'a>;
}


pub struct ConnectionPool {
    db_pool: DatabaseConnectionPool,
    cache_pool: CacheConnectionPool,
}


/// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct Context {
    pub db: DatabaseConnection,
    pub cache: CacheConnection,
}


/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for Context {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Context, ()> {
        let pool = request.guard::<State<ConnectionPool>>()?;
        let db_conn = match pool.db_pool.get() {
            Ok(conn) => conn,
            Err(_) => {return Outcome::Failure((Status::ServiceUnavailable, ()))}
        };
        let cache_conn = match pool.cache_pool.get() {
            Ok(conn) => conn,
            Err(_) => {return Outcome::Failure((Status::ServiceUnavailable, ()))}
        };
        Outcome::Success(Context{db: db_conn, cache: cache_conn})
    }
}


impl Deref for Context {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}


/// Initializes a database pool.
pub fn init_pool() -> ConnectionPool {
    let db_config = r2d2::Config::default();
    let cache_config = r2d2::Config::default();
    let db_manager = PgConnectionManager::new(DATABASE_URL);
    let cache_manager = RedisConnectionManager::new("redis://localhost").expect("cache connection");
    ConnectionPool {
        db_pool: r2d2::Pool::new(db_config, db_manager).expect("db pool"),
        cache_pool: r2d2::Pool::new(cache_config, cache_manager).expect("cache pool")
    }
}
