use diesel::Insertable;
use rust_order_api::schema::{users, products};
use serde::{Deserialize, Serialize};

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name=users)]
pub struct NewUser {
    pub username: String,
}

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name=products)]
pub struct NewProduct {
    pub title: String,
    pub category_id: i32,
    pub author: String,
    pub list_price: f64,
    pub stock_quantity: i32,
}
