use diesel;
use diesel::ExecuteDsl;
use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_s3::{S3, S3Client, HeadObjectRequest, CopyObjectRequest, GetObjectRequest,
                 PutObjectRequest, DeleteObjectRequest, PutBucketCorsRequest, CORSConfiguration,
                 CORSRule, CreateBucketRequest, DeleteBucketRequest, CreateMultipartUploadRequest,
                 UploadPartRequest, CompleteMultipartUploadRequest, CompletedMultipartUpload,
                 CompletedPart, ListObjectsV2Request};
use rusoto_core::default_tls_client;

use hyper;
use rusoto_core::ProvideAwsCredentials;


#[derive(Debug,Serialize,Deserialize)]
pub struct ResourceStorageError {}


pub trait ResourceStorage<Form, Model, Filters>{
    fn create<'a>(&self, form: Form)
        -> Result<Model, ResourceStorageError>;

    fn bulk_create<'a>(&self, form: Vec<Form>)
        -> Result<(), ResourceStorageError>;

    fn list<'a>(&self, filters: Filters)
        -> Result<Vec<Model>, ResourceStorageError>;
}


pub struct RedshiftIngest<'a> {
    pub base_bucket: String,
    pub acl: Option<String>,
    s3_client: S3Client<DefaultCredentialsProvider, hyper::Client>,
    pg_client: &'a diesel::pg::PgConnection,
}


impl<'a> RedshiftIngest<'a> {
    pub fn new(pg_client: &'a diesel::pg::PgConnection) -> Self {
        Self {
            base_bucket: "".to_owned(),
            acl: Some("".to_owned()),
            s3_client: S3Client::new(default_tls_client().unwrap(),
                                     DefaultCredentialsProvider::new().unwrap(),
                                     Region::UsEast1),
            pg_client: pg_client,
        }
    }

    pub fn upload_to_s3(&self, path: String, stream: Vec<u8>) -> Result<String, ()> {
        let path_split: Vec<&str> = path.split("/").collect();

        let file_name = path_split[path_split.len() - 1];
        let base_path = path_split[0..path_split.len() - 1].join("");

        let result = self.s3_client.put_object(&PutObjectRequest {
            acl: self.acl.clone(),
            body: Some(stream),
            bucket: format!("{}/{}", self.base_bucket.to_owned(), base_path),
            key: file_name.to_owned(),
            ..Default::default()
        });
        Ok(format!("{}/{}", self.base_bucket, file_name).to_owned())

    }

    pub fn ingest_from_s3(&self, table_name: String, path: String) -> Result<usize, ()> {
        let location = format!("s3://{}", path);
        let credentials = DefaultCredentialsProvider::new().unwrap().credentials().unwrap();
        let key = credentials.aws_access_key_id();
        let secret = credentials.aws_secret_access_key();
        let credentials = format!("credentials aws_secret_access_key={};aws_secret_access_key={}", key, secret);
        let query = diesel::expression::dsl::sql::<diesel::types::Bool>(
            format!("copy {} from '{}' {} csv;", table_name, location, credentials).as_str()
        );
        match query.execute(&*self.pg_client) {
            Ok(count) => Ok(count),
            Err(_) => Err(()),
        }
    }

    pub fn process(&self, table_name: String, stream: Vec<u8>) {
        let uploaded_file = self.upload_to_s3(format!("{}/{}", table_name, "0.csv"), stream)
                                .expect("Failed to upload to S3.");
        let ingested = self.ingest_from_s3(table_name, uploaded_file)
                           .expect("Failed to ingest to Redshift.");
    }
}



