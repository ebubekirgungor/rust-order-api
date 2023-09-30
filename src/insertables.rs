use diesel::Insertable;
use rust_order_api::schema::{campaigns, orders, products, users};
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

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name=orders)]
pub struct NewOrder {
    pub price_without_discount: f64,
    pub discounted_price: f64,
    pub campaign_id: Option<i32>,
    pub user_id: i32,
}
