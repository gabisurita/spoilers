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
