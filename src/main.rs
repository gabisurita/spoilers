#![feature(plugin, decl_macro, type_ascription, i128_type)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;


#[cfg(test)] mod tests;

use std::thread;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use serde_json::Value;
use rocket_contrib::{Json, JsonValue};
use rocket::State;
use std::collections::HashMap;
use std::sync::Mutex;
use postgres::types::{Type, FromSql, ToSql};
use postgres::rows::{Rows, Row};
use postgres::stmt::Column;
use postgres::error::Error as PgError;


#[derive(Debug)]
enum StorageError {
    PgError
}


impl From<PgError> for StorageError {
    fn from(err: PgError) -> StorageError {
        StorageError::PgError
    }
}



struct Context {
    pub storage: r2d2::Pool<PostgresConnectionManager>
}

type ContextWrapper = Mutex<Context>;


#[post("/", format = "application/json", data = "<message>")]
fn new(message: Json<Value>, context: State<ContextWrapper>) -> JsonValue {
    JsonValue(json!({
        "request": message.0,
        "context": context.lock().unwrap().storage.get().unwrap().execute("SELECT * FROM hello_world;", &[]).unwrap(),
    }))
}


fn convert(row: &Row, column: &Column) -> Value {
    println!("{}", column.type_().name());
    let name = column.name();
    match column.type_().name() {
        "int4" => json!(row.get(name): i32),
        "varchar" => json!(row.get(name): String),
        "text" => json!(row.get(name): String),
        _ => json!(row.get_bytes(name))
    }
}


fn unwrap_records(query_result: Result<Rows, PgError>) -> Result<Vec<HashMap<String, Value>>, StorageError> {
    let result = try!(query_result);
    Ok(result.iter().map(|r|
        result.columns().iter().map(|c|
            (c.name().to_owned(), convert(&r, c))
        ).collect()
    ).collect())
}


#[get("/<table_name>/records", format = "application/json")]
fn get(table_name: String, context: State<ContextWrapper>) -> JsonValue {
    let storage = context.lock().unwrap().storage.get().unwrap();
    let query = format!("SELECT * FROM {table_name};", table_name=table_name.as_str());
    let result = storage.query(query.as_str(), &[]);
    let records = unwrap_records(result);
    match records {
        Ok(data) => JsonValue(json!({"data": data})),
        Err(err) => JsonValue(json!({"errors": format!("{:?}", err)}))
    }
}


#[post("/<table_name>/records", format = "application/json", data = "<record>")]
fn create<'a>(table_name: String, record: Json<Value>, context: State<ContextWrapper>) -> JsonValue {
    let storage = context.lock().unwrap().storage.get().unwrap();
    let fields: Vec<String> = record.0.as_object().unwrap().keys().map(|s| s.clone()).collect();
    let holders: Vec<String> = (1..fields.len()+1).map(|i| format!("${}", i)).collect();
    let query = format!("INSERT INTO {table_name} ({fields}) VALUES ({values});",
                        table_name=table_name.as_str(),
                        fields=fields.join(", "),
                        values=holders.join(", "));

    let mut values: Vec<&ToSql> = vec![];
    for v in record.0.as_object().unwrap().values().map(|v| v.as_str()) {
        values.push(&v.unwrap().to_owned(): &'a String);
    }

    let result = storage.execute(query.as_str(), &values[0..fields.len()]).unwrap();
    JsonValue(json!({"data": record.0}))
}

//#[put("/<id>", format = "application/json", data = "<message>")]
//fn update(id: ID, message: Json<Message>, map: State<MessageMap>) -> Option<JsonValue> {
//    let mut hashmap = map.lock().unwrap();
//    if hashmap.contains_key(&id) {
//        hashmap.insert(id, message.0.contents);
//        Some(json!({ "status": "ok" }))
//    } else {
//        None
//    }
//}
//
//#[get("/<id>", format = "application/json")]
//fn get(id: ID, map: State<MessageMap>) -> Option<Json<Message>> {
//    let hashmap = map.lock().unwrap();
//    hashmap.get(&id).map(|contents| {
//        Json(Message {
//            id: Some(id),
//            contents: contents.clone()
//        })
//    })
//}

#[catch(404)]
fn not_found() -> JsonValue {
    JsonValue(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

fn main() {

    let config = r2d2::Config::default();
    let manager = PostgresConnectionManager::new("postgres://gsurita@localhost",
                                                 TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();
    let context = ContextWrapper::new(
        Context {
            storage: pool,
        },
    );

    let server = rocket::ignite()
        .mount("/", routes![new, get, create])
        .catch(catchers![not_found])
        .manage(context);
    server.launch();
}
