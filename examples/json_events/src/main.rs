#![feature(plugin, custom_attribute, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate rocket_codegen;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use] extern crate spoilers;
#[macro_use] extern crate spoilers_derive;

use diesel::*;
use serde_json::Value;
use rocket_contrib::{Json, JsonValue};

use spoilers::models::*;
use spoilers::storage::*;

use chrono::NaiveDateTime;


// Declare your table types here

table! {
    events {
        id -> Integer,
        timestamp -> Timestamp,
        body -> Nullable<Jsonb>,
    }
}

// Declare your models here

#[derive(Resource, PgStorageBackend, CollectionGet, CollectionCreate)]
#[endpoint="/"]
#[table_name="events"]
pub struct Event {
    pub timestamp: NaiveDateTime,
    pub body: Option<serde_json::Value>,
}


#[catch(404)]
fn not_found() -> JsonValue {
    JsonValue(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}


fn main() {
    let server = rocket::ignite()
        .mount("/", routes![event_create, event_get])
        .catch(catchers![not_found])
        .manage(init_pool());
    server.launch();
}
