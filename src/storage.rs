#[derive(Debug,Serialize,Deserialize)]
pub struct ResourceStorageError {}


pub trait ResourceStorage<Form, Model, Filters>{
    fn create<'a>(&self, form: Form)
        -> Result<Model,ResourceStorageError>;

    fn bulk_create<'a>(&self, form: Vec<Form>)
        -> Result<Model,ResourceStorageError>;

    fn list<'a>(&self, filters: Filters)
        -> Result<Vec<Model>,ResourceStorageError>;
}
