#![feature(plugin, custom_attribute, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate spoilers;
#[macro_use] extern crate spoilers_derive;
extern crate redis;

use std::time::Duration;

use diesel::*;
use spoilers::*;
use spoilers::models::*;
use spoilers::storage::*;

use chrono::NaiveDateTime;


#[derive(RedshiftStorage)]
pub struct Postgres {}



// Declare your table types here

table! {
    events {
        id -> Integer,
        timestamp -> Timestamp,
        body -> Nullable<Jsonb>,
    }
}

// Declare your models here

#[derive(Resource, RedshiftResourceStorage, CollectionGet, CollectionCreate)]
#[endpoint="/"]
#[table_name="events"]
pub struct Event {
    pub timestamp: NaiveDateTime,
    pub body: Option<serde_json::Value>,
}


#[catch(404)]
fn not_found() -> rocket_contrib::JsonValue {
    rocket_contrib::JsonValue(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}





fn main() {
    let server_pool = Postgres::init_pool();
    let server = rocket::ignite()
        .mount("/", routes![event_create, event_get])
        .catch(catchers![not_found])
        .manage(Postgres::init_pool());


    let async_pool = Postgres::init_pool();
    Event::sync(async_pool, Duration::new(15*60, 0));
    server.launch();
}
