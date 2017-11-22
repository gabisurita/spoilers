pub trait Resource {
    const ENDPOINT: &'static str;
}


pub trait CollectionGet: Resource {
    fn collection_get(&self);
}


pub trait CollectionCreate: Resource {
    fn collection_post(&self);
}


//trait CollectionDelete: Resource {
//    fn collection_delete(&self);
//}
//
//
//trait RecordGet: Resource {
//    fn record_get(&self);
//}
//
//
//trait RecordUpate: Resource {
//    fn record_update(&self);
//}
//
//
//trait RecordPatch: Resource {
//    fn record_patch(&self);
//}
//
//
//trait RecordDelete: Resource {
//    fn record_delete(&self);
//}
