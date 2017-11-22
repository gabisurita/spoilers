#![feature(plugin, decl_macro, type_ascription, custom_attribute)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate r2d2;
extern crate r2d2_diesel;

pub mod models;
pub mod storage;
#[cfg(test)] mod tests;


static DATABASE_URL: &'static str = env!("DATABASE_URL");


macro_rules! create_endpoint {
    ($endpoint:expr, $method_name:ident, $table_name:ident, $model_type:ident, $form_type:ident) => (
        #[post($endpoint, format = "application/json", data = "<message>")]
        fn $method_name(message: Json<Value>, conn: DbConn) -> JsonValue {
            let new: $form_type = serde_json::from_value(message.0).unwrap();
            let created: $model_type = diesel::insert(&new).into($table_name::table)
                .get_result(&*conn)
                .expect("Error saving new post");
            JsonValue(json!({"data": created}))
        }
    )
}


macro_rules! list_endpoint {
    ($endpoint:expr, $method_name:ident, $table_name:ident, $model_type:ident) => (
        #[get($endpoint, format = "application/json")]
        fn $method_name(conn: DbConn) -> JsonValue {
            use self::models::$table_name::dsl::*;
            let results = events.limit(10)
                .load::<$model_type>(&*conn)
                .expect("Error loading events");
            JsonValue(json!({"data": results}))
        }
    )
}
