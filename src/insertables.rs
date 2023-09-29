use rust_order_api::schema::users;
use diesel::Insertable;
use serde::{Serialize, Deserialize};

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name=users)]
pub struct NewUser {
  pub username: String,
}