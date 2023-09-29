use diesel::Insertable;
use rust_order_api::schema::{campaigns, products, users};
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

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name=campaigns)]
pub struct NewCampaign {
    pub description: String,
    pub min_purchase_price: f64,
    pub min_purchase_quantity: i32,
    pub discount_quantity: i32,
    pub discount_percent: i32,
    pub rule_author: String,
    pub rule_category: String,
}
