#![feature(plugin, decl_macro, type_ascription, custom_attribute)]

pub extern crate chrono;
#[macro_use] pub extern crate diesel;
#[macro_use] pub extern crate diesel_codegen;
pub extern crate hyper;
pub extern crate rocket;
pub extern crate rocket_contrib;
pub extern crate serde;
#[macro_use] pub extern crate serde_derive;
#[macro_use] pub extern crate serde_json;
pub extern crate redis;
extern crate rusoto_core;
extern crate rusoto_s3;
pub extern crate r2d2;
pub extern crate r2d2_diesel;
pub extern crate r2d2_redis;

pub mod models;
pub mod storage;
#[cfg(test)] mod tests;
