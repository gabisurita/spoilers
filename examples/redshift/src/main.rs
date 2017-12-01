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
extern crate csv;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;


use std::time::Duration;

use diesel::*;
use spoilers::*;
use spoilers::models::*;
use spoilers::storage::*;

use chrono::NaiveDateTime;

use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_s3::{S3, S3Client, HeadObjectRequest, CopyObjectRequest, GetObjectRequest,
                 PutObjectRequest, DeleteObjectRequest, PutBucketCorsRequest, CORSConfiguration,
                 CORSRule, CreateBucketRequest, DeleteBucketRequest, CreateMultipartUploadRequest,
                 UploadPartRequest, CompleteMultipartUploadRequest, CompletedMultipartUpload,
                 CompletedPart, ListObjectsV2Request};
use rusoto_core::default_tls_client;

use hyper::Client;
use rusoto_core::ProvideAwsCredentials;


#[derive(RedshiftStorage)]
pub struct Redshift {}

impl Redshift {
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


// Declare your table types here

table! {
    log_level_critical (id) {
        id -> Int4,
        timestamp -> Nullable<Timestamp>,
        user_id -> Nullable<Int4>,
        title -> Nullable<Varchar>,
        body -> Nullable<Text>,
    }
}

table! {
    log_level_warning (id) {
        id -> Int4,
        timestamp -> Nullable<Timestamp>,
        user_id -> Nullable<Int4>,
        title -> Nullable<Varchar>,
        body -> Nullable<Text>,
    }
}

// Declare your models here

#[derive(Resource, RedshiftResourceStorage, Debug)]
#[endpoint="/"]
#[table_name="log_level_warning"]
pub struct Warning {
    pub timestamp: Option<NaiveDateTime>,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Resource, RedshiftResourceStorage, Debug)]
#[endpoint="/"]
#[table_name="log_level_critical"]
pub struct Error {
    pub timestamp: Option<NaiveDateTime>,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub body: Option<String>,
}


// Declare your routes here and syncs

fn main() {
    let credentials = DefaultCredentialsProvider::new().unwrap();
    println!("{:#?}", credentials.credentials());

    let client = S3Client::new(default_tls_client().unwrap(),
                               DefaultCredentialsProvider::new().unwrap(),
                               Region::UsEast1);

    let bucket = "spoilers-development".to_owned();

    //let create_bucket_req = CreateBucketRequest {
    //    bucket: bucket,
    //    ..Default::default()
    //};

    //let result = client.create_bucket(&create_bucket_req).expect("Couldn't create bucket");
    //println!("{:#?}", result);

    //test_multipart_upload(&client, bucket.as_ref(), "test-upload.txt");
    //test_delete_object(&client, bucket.as_ref(), "test-upload.txt");

    //let list_obj_req = ListObjectsV2Request {
    //    bucket: bucket.to_owned(),
    //    ..Default::default()
    //};
    //let result = client.list_objects_v2(&list_obj_req).expect("Couldn't list items in bucket (v2)");
    //println!("Items in bucket: {:#?}", result);

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
