use syn;
use quote;


pub fn impl_postgre_storage(ast: &syn::DeriveInput) -> quote::Tokens {
    let class_name = &ast.ident;

    quote! {
        type DatabaseConnectionPool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>;
        type DatabaseConnection = r2d2::PooledConnection<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>;


        pub struct ConnectionPool {
            db_pool: DatabaseConnectionPool,
        }


        /// Connection request guard type: a wrapper around an r2d2 pooled connection.
        pub struct Context {
            pub db: DatabaseConnection,
        }


        /// Attempts to retrieve a single connection from the managed database pool. If
        /// no pool is currently managed, fails with an `InternalServerError` status. If
        /// no connections are available, fails with a `ServiceUnavailable` status.
        impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for Context {
            type Error = ();

            fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Context, ()> {
                let pool = request.guard::<rocket::State<ConnectionPool>>()?;
                let db_conn = match pool.db_pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {
                        return rocket::Outcome::Failure(
                            (rocket::http::Status::ServiceUnavailable, ())
                        )
                    }
                };
                rocket::Outcome::Success(Context{db: db_conn})
            }
        }

        impl #class_name {
            /// Initializes a database pool.
            pub fn init_pool() -> ConnectionPool {
                use std::env;
                let db_config = r2d2::Config::default();
                let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
                let db_manager = r2d2_diesel::ConnectionManager::new(database_url);
                ConnectionPool {
                    db_pool: r2d2::Pool::new(db_config, db_manager).expect("db pool"),
                }
            }
        }
    }
}


pub fn impl_redshift_storage(ast: &syn::DeriveInput) -> quote::Tokens {
    let class_name = &ast.ident;

    quote! {
        type DatabaseConnectionPool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>;
        type DatabaseConnection = r2d2::PooledConnection<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>;
        type QueueConnectionPool = r2d2::Pool<r2d2_redis::RedisConnectionManager>;
        type QueueConnection = r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>;



        pub struct ConnectionPool {
            db_pool: DatabaseConnectionPool,
            queue_pool: QueueConnectionPool,
        }


        /// Connection request guard type: a wrapper around an r2d2 pooled connection.
        pub struct Context {
            pub db: DatabaseConnection,
            pub queue: QueueConnection,
        }


        /// Attempts to retrieve a single connection from the managed database pool. If
        /// no pool is currently managed, fails with an `InternalServerError` status. If
        /// no connections are available, fails with a `ServiceUnavailable` status.
        impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for Context {
            type Error = ();

            fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Context, ()> {
                let pool = request.guard::<rocket::State<ConnectionPool>>()?;
                let db_conn = match pool.db_pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {
                        return rocket::Outcome::Failure(
                            (rocket::http::Status::ServiceUnavailable, ())
                        )
                    }
                };
                let queue_conn = match pool.queue_pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {
                        return rocket::Outcome::Failure(
                            (rocket::http::Status::ServiceUnavailable, ())
                        )
                    }
                };
                rocket::Outcome::Success(Context{db: db_conn, queue: queue_conn})
            }
        }

        impl #class_name {
            /// Initializes a database pool.
            pub fn init_pool() -> ConnectionPool {
                use std::env;

                let db_config = r2d2::Config::default();
                let queue_config = r2d2::Config::default();

                let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
                let queue_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

                let db_manager = r2d2_diesel::ConnectionManager::new(database_url);

                let queue_manager = r2d2_redis::RedisConnectionManager::new(queue_url.as_ref())
                    .expect("Redis connection failed");

                ConnectionPool {
                    db_pool: r2d2::Pool::new(db_config, db_manager)
                                         .expect("db pool"),
                    queue_pool: r2d2::Pool::new(queue_config, queue_manager)
                                            .expect("redis pool"),
                }
            }
        }
    }
}
