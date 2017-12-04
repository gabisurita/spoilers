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
    pub acl: Option<String>,
    s3_client: S3Client<DefaultCredentialsProvider, hyper::Client>,
    pg_client: &'a diesel::pg::PgConnection,
}


impl<'a> RedshiftIngest<'a> {
    pub fn new(pg_client: &'a diesel::pg::PgConnection) -> Self {
        Self {
            acl: Some("public-read".to_owned()),
            s3_client: S3Client::new(default_tls_client().unwrap(),
                                     DefaultCredentialsProvider::new().unwrap(),
                                     Region::UsEast1),
            pg_client: pg_client,
        }
    }

    pub fn upload_to_s3(&self, s3_path: String, stream: Vec<u8>) -> Result<String, ()> {
        let file_name = "0.csv".to_owned();

        let result = self.s3_client.put_object(&PutObjectRequest {
            acl: self.acl.clone(),
            body: Some(stream),
            bucket: s3_path.clone(),
            key: file_name.to_owned(),
            ..Default::default()
        });
        Ok(format!("{}/{}", s3_path, file_name).to_owned())

    }

    pub fn ingest_from_s3(&self, table_name: String, path: String) -> Result<usize, ()> {
        let location = format!("s3://{}", path);
        let credentials = DefaultCredentialsProvider::new().unwrap().credentials().unwrap();
        let key = credentials.aws_access_key_id();
        let secret = credentials.aws_secret_access_key();
        let credentials = format!("credentials 'aws_access_key_id={};aws_secret_access_key={}'", key, secret);
        let command = format!("copy {} from '{}' {} csv;", table_name, location, credentials);
        println!("{}", command);
	let query = diesel::expression::dsl::sql::<diesel::types::Bool>(
            command.as_str()
	);
        match query.execute(&*self.pg_client) {
            Ok(count) => Ok(count),
            Err(e) => {println!("{:?}", e); Err(())},
        }
    }

    pub fn process(&self, table_name: String, base_path: String, stream: Vec<u8>) {
        let uploaded_file = self.upload_to_s3(base_path, stream)
                                .expect("Failed to upload to S3.");
        let ingested = self.ingest_from_s3(table_name, uploaded_file)
                           .expect("Failed to ingest to Redshift.");
    }
}



