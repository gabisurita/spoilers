use std::ops::Deref;

use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::{State, Request, Outcome};
use rocket::request::{self, FromRequest};
use rocket::http::Status;

use super::DATABASE_URL;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Initializes a database pool.
pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
    r2d2::Pool::new(config, manager).expect("db pool")
}
