#![feature(plugin, custom_attribute, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate spoilers;
#[macro_use] extern crate spoilers_derive;

use diesel::*;
use spoilers::*;
use spoilers::models::*;
use spoilers::storage::*;

use chrono::NaiveDateTime;


#[derive(PostgreStorage)]
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

#[derive(Resource, PgResourceStorage, CollectionGet, CollectionCreate)]
#[endpoint="/"]
#[table_name="events"]
pub struct Event {
    pub timestamp: NaiveDateTime,
    pub body: Option<serde_json::Value>,
}


// Declare your routes here

fn main() {
    let server_pool = Postgres::init_pool();
    let server = rocket::ignite()
        .mount("/", routes![event_create, event_get])
        .manage(server_pool);
    server.launch();
}
