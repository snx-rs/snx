use diesel::{prelude::Queryable, Selectable};
use serde::Serialize;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}
