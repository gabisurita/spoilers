#![feature(attr_literals)]
#![recursion_limit="256"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;
extern crate spoilers;


use std::collections::HashMap;
use proc_macro::TokenStream;


/// Util to parse a derive input from a derive decored token.
fn parse_derive_input(input: TokenStream) -> syn::DeriveInput {
    let s = input.to_string();
    syn::parse_derive_input(&s).unwrap()
}


/// Util to parse derive attributes from a derive input.
fn parse_derive_attibutes<'a>(ast: syn::DeriveInput) -> HashMap<String, syn::MetaItem> {
    ast.attrs.iter().map(|x| (x.name().to_owned(), x.value.clone())).collect()
}

/// State composed of macro variables used as an util to generate
/// macro implementations.
struct MetaResourceConfig {
    pub ast: syn::DeriveInput,
}


impl MetaResourceConfig {
    pub fn new(input: syn::DeriveInput) -> MetaResourceConfig {
        MetaResourceConfig {ast: input}
    }

    pub fn endpoint(&self) -> String {
        let attr_items = parse_derive_attibutes(self.ast.clone());
        attr_items.get("endpoint").map_or(
            self.struct_name().as_ref().to_owned(),
            |attr| match attr {
                &syn::MetaItem::NameValue(_, ref literal) => {
                    match literal {
                        &syn::Lit::Str(ref path, _) => path.to_owned(),
                        _ => panic!("Endpoint must be a string")
                    }
                },
                _ => panic!("Endpoint must be a string")
            }
        )
    }

    pub fn table_name(&self) -> syn::Ident {
        let attr_items = parse_derive_attibutes(self.ast.clone());
        syn::Ident::new(
            attr_items.get("table_name").map_or(
                self.struct_name().as_ref(),
                |attr| match attr {
                    &syn::MetaItem::NameValue(_, ref literal) => {
                        match literal {
                            &syn::Lit::Str(ref path, _) => path,
                            _ => panic!("Endpoint must be a string")
                        }
                    },
                    _ => panic!("Endpoint must be a string")
                }
            )
        )
    }

    pub fn struct_name(&self) -> syn::Ident {
        self.ast.ident.clone()
    }

    pub fn model_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Model", self.struct_name()))
    }

    pub fn form_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Form", self.struct_name()))
    }

    pub fn filter_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Filter", self.struct_name()))
    }

    pub fn method_name(&self, verb: &str) -> syn::Ident {
        let ref_name = self.struct_name().as_ref().to_lowercase();
        syn::Ident::new(format!("{}_{}", ref_name, verb))
    }

    pub fn fields(&self) -> &[syn::Field] {
        match self.ast.body {
            syn::Body::Struct(ref data) => data.fields(),
            _ => panic!("Resources must be structs.")
        }
    }
}


#[proc_macro_derive(Resource, attributes(endpoint))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    impl_resource(&parse_derive_input(input)).parse().unwrap()
}


#[proc_macro_derive(CollectionGet, attributes(endpoint))]
pub fn derive_collection_get(input: TokenStream) -> TokenStream {
    impl_collection_get(&parse_derive_input(input)).parse().unwrap()
}


#[proc_macro_derive(CollectionCreate, attributes(endpoint))]
pub fn derive_collection_create(input: TokenStream) -> TokenStream {
    impl_collection_create(&parse_derive_input(input)).parse().unwrap()
}


fn impl_resource(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let model_name = config.model_name();
    let form_name = config.form_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name().as_ref().to_owned();
    let endpoint = config.endpoint().to_owned();

    let model_fields: Vec<quote::Tokens> = config.fields().iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote!{
            pub #ident: #ty,
        }
    }).collect();
    let form_fields: Vec<quote::Tokens> = config.fields().iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote!{
            pub #ident: #ty,
        }
    }).collect();

    quote! {
        #[derive(Queryable, Serialize, Deserialize)]
        pub struct #model_name {
            pub id: i32,
            #(#model_fields)*
        }

        #[derive(Insertable, Serialize, Deserialize)]
        #[table_name=#table_name]
        pub struct #form_name {
            #(#form_fields)*
        }

        #[derive(Serialize, Deserialize)]
        pub struct #filter_name {
        }

        impl Resource for #struct_name {
            const ENDPOINT: &'static str = #endpoint;
        }
    }
}


fn impl_collection_get(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let endpoint = config.endpoint().to_owned();
    let method_name = config.method_name("get");
    let struct_name = config.struct_name();
    let filter_name = config.filter_name();

    quote! {
        #[get(#endpoint, format = "application/json")]
        fn #method_name(context: Context) -> rocket_contrib::JsonValue {
            let data = context.list(#filter_name {});
            rocket_contrib::JsonValue(json!({"data": data.expect("error")}))
        }
    }
}


fn impl_collection_create(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let endpoint = config.endpoint().to_owned();
    let method_name = config.method_name("create");
    let struct_name = config.struct_name();
    let form_name = config.form_name();

    quote! {
        #[post(#endpoint, format = "application/json", data = "<message>")]
        fn #method_name(message: rocket_contrib::Json<serde_json::Value>, context: Context)
                -> rocket_contrib::JsonValue {
            let new: #form_name = serde_json::from_value(message.0).unwrap();
            let created = context.create(new);
            rocket_contrib::JsonValue(json!({"data": created.expect("error")}))
        }
    }
}


#[proc_macro_derive(PgResourceStorage, attributes(endpoint, table_name))]
pub fn derive_pg_storage_backend(input: TokenStream) -> TokenStream {
    impl_pg_storage_backend(&parse_derive_input(input)).parse().unwrap()
}


fn impl_pg_storage_backend(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let form_name = config.form_name();
    let model_name = config.model_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name();

    quote! {

        impl spoilers::storage::ResourceStorage<#form_name,#model_name,#filter_name>
                for Context {

            fn create<'a>(&self, form: #form_name) ->
                    Result<#model_name, ResourceStorageError> {

                let created: #model_name = diesel::insert(&form).into(#table_name::table)
                    .get_result(&*self.db)
                    .expect("Error saving new post");
                Ok(created)
            }

            fn bulk_create<'a>(&self, form: Vec<#form_name>) ->
                    Result<#model_name, ResourceStorageError> {

                let created: #model_name = diesel::insert(&form).into(#table_name::table)
                    .get_result(&*self.db)
                    .expect("Error saving new post");
                Ok(created)
            }

            fn list<'a>(&self, filters: #filter_name) ->
                    Result<Vec<#model_name>, ResourceStorageError> {

                use #table_name::dsl::*;
                let results = #table_name.limit(10)
                    .load::<#model_name>(&*self.db)
                    .expect("Error loading events");
                Ok(results)
            }
        }
    }
}


#[proc_macro_derive(PostgreStorage)]
pub fn derive_postgre_storage(input: TokenStream) -> TokenStream {
    impl_postgre_storage(&parse_derive_input(input)).parse().unwrap()
}


fn impl_postgre_storage(ast: &syn::DeriveInput) -> quote::Tokens {
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


#[proc_macro_derive(RedshiftStorage)]
pub fn derive_redshift_storage(input: TokenStream) -> TokenStream {
    impl_redshift_storage(&parse_derive_input(input)).parse().unwrap()
}



fn impl_redshift_storage(ast: &syn::DeriveInput) -> quote::Tokens {
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



#[proc_macro_derive(RedshiftResourceStorage, attributes(endpoint, ))]
pub fn derive_redshift_storage_backend(input: TokenStream) -> TokenStream {
    impl_redshift_storage_backend(&parse_derive_input(input)).parse().unwrap()
}


fn impl_redshift_storage_backend(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let form_name = config.form_name();
    let model_name = config.model_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name();
    let queue_name = config.table_name().as_ref().to_owned();

    quote! {
        use std::time::Duration as LocalDuration;
        impl #struct_name {
            pub fn sync(pool: ConnectionPool, period: LocalDuration) {
                use std::{thread, time};
                use redis::Commands;
                let db_conn = match pool.db_pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {return;}
                };
                let queue_conn = match pool.queue_pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {return;}
                };

                let context = Context {
                    db: db_conn,
                    queue: queue_conn,
                };

                let th = thread::spawn(move || {
                    loop {
                        let cached: Vec<String> = context.queue.lrange(#queue_name, 0, -1)
                                                               .unwrap_or_default();


                        let cache_results: Vec<#form_name> = cached.iter().map(|s| {
                            serde_json::from_str(s.as_ref()).unwrap()
                        }).collect();

                        if cache_results.len() > 0 {
                            context.bulk_create(cache_results).unwrap();
                            let _: i32 = context.queue.del(#queue_name).unwrap();
                        }
                        thread::sleep(period);
                    }
                });
            }
        }

        impl spoilers::storage::ResourceStorage<#form_name,#model_name,#filter_name>
                for Context {

            fn create<'a>(&self, form: #form_name) ->
                    Result<#model_name, ResourceStorageError> {
                use redis::Commands;
                let serialized: String = serde_json::to_string(&form).unwrap();
                let _: i32 = self.queue.rpush(#queue_name, serialized.clone()).unwrap();

                let mut model_json: serde_json::Value =
                    serde_json::from_str(serialized.as_ref()).unwrap();

                model_json["id"] = json!(-1);
                let result: #model_name = serde_json::from_value(model_json).unwrap();
                Ok(result)
            }

            fn bulk_create<'a>(&self, form: Vec<#form_name>) ->
                    Result<#model_name, ResourceStorageError> {
                let created: #model_name = diesel::insert(&form).into(#table_name::table)
                    .get_result(&*self.db)
                    .expect("Error saving new post");
                Ok(created)
            }

            fn list<'a>(&self, filters: #filter_name) ->
                    Result<Vec<#model_name>, ResourceStorageError> {

                use redis::Commands;
                use #table_name::dsl::*;
                let mut db_results = #table_name
                    .limit(1000)
                    .load::<#model_name>(&*self.db)
                    .expect("Error loading events");
                let cached: Vec<String> = self.queue.lrange(#queue_name, 0, -1).unwrap();
                let cache_results: Vec<#model_name> = cached.iter().map(|s| {
                    let mut model_json: serde_json::Value =
                        serde_json::from_str(s.as_ref()).unwrap();
                    model_json["id"] = json!(-1);
                    serde_json::from_value(model_json).unwrap()
                }).collect();
                db_results.extend(cache_results);
                Ok(db_results)
            }
        }
    }
}



fn impl_redis_storage_backend(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let form_name = config.form_name();
    let model_name = config.model_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name().as_ref().to_owned();

    quote! {

        impl spoilers::storage::ResourceStorage<#form_name,#model_name,#filter_name,> for #struct_name {

            fn create<'a>(form: #form_name, conn: &'a redis::Connection) ->
                    Result<#model_name, ResourceStorageError> {
                let serialized: String = serde::serialize(&form).unwrap();
                let id: i32 = conn.radd(#table_name, serialized);
                let model: serde::from_value(serialized).unwrap();
                Ok(model)
            }

            fn bulk_create<'a>(form: Vec<#form_name>, conn: &'a redis::Connection) ->
                    Result<#model_name, ResourceStorageError> {
            }

            fn list<'a>(filters: #filter_name, conn: &'a redis::Connection) ->
                    Result<Vec<#model_name>, ResourceStorageError> {
                Ok(vec![])
            }
        }
    }
}
