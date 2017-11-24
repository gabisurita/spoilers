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

use std::time::Duration;

use diesel::*;
use spoilers::*;
use spoilers::models::*;
use spoilers::storage::*;

use chrono::NaiveDateTime;


#[derive(RedshiftStorage)]
pub struct Redshift {}


// Declare your table types here

table! {
    log_level_critical (id) {
        id -> Int4,
        timestamp -> Timestamp,
        user_id -> Nullable<Int4>,
        title -> Nullable<Varchar>,
        body -> Nullable<Text>,
    }
}

table! {
    log_level_warning (id) {
        id -> Int4,
        timestamp -> Timestamp,
        user_id -> Nullable<Int4>,
        title -> Nullable<Varchar>,
        body -> Nullable<Text>,
    }
}

// Declare your models here

#[derive(Resource, RedshiftResourceStorage, CollectionGet, CollectionCreate)]
#[endpoint="/"]
#[table_name="log_level_warning"]
pub struct Warning {
    pub timestamp: NaiveDateTime,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Resource, RedshiftResourceStorage, CollectionGet, CollectionCreate)]
#[endpoint="/"]
#[table_name="log_level_critical"]
pub struct Error {
    pub timestamp: NaiveDateTime,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub body: Option<String>,
}


// Declare your routes here and syncs

fn main() {
    let server_pool = Redshift::init_pool();
    let async_pool = Redshift::init_pool();

    let server = rocket::ignite()
        .mount("/warning", routes![warning_create, warning_get])
        .mount("/error", routes![error_create, error_get])
        .manage(server_pool);

    Warning::sync(&async_pool, Duration::new(30 * 60, 0));
    Error::sync(&async_pool, Duration::new(10 * 60, 0));
    server.launch();

}
