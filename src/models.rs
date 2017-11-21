use serde_json;
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

#[derive(Queryable, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub body: Option<serde_json::Value>,
}

// Declare your forms here

#[derive(Insertable, Serialize, Deserialize)]
#[table_name="events"]
pub struct EventForm {
    pub timestamp: NaiveDateTime,
    pub body: Option<serde_json::Value>,
}
