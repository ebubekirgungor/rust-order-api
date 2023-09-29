use diesel::prelude::*;
use serde::Serialize;
use crate::schema::{users, categories, products, orders, campaigns, orders_products};

#[derive(Serialize, Queryable, Selectable, Insertable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub title: String,
}

#[derive(Serialize, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Category))]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub category_id: i32,
    pub author: String,
    pub list_price: f64,
    pub stock_quantity: i32,
}

#[derive(Serialize, Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = campaigns)]
pub struct Campaign {
    pub id: i32,
    pub description: String,
    pub min_purchase_price: Option<f64>,
    pub min_purchase_quantity: Option<i32>,
    pub discount_quantity: Option<i32>,
    pub discount_percent: Option<i32>,
    pub rule_author: Option<String>,
    pub rule_category: Option<String>,
}

#[derive(Serialize, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Campaign))]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub price_without_discount: f64,
    pub discounted_price: f64,
    pub campaign_id: i32,
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(Order))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = orders_products)]
#[diesel(primary_key(order_id, product_id))]
pub struct OrderToProduct {
    pub order_id: i32,
    pub product_id: i32,
}
