use diesel::prelude::*;
use crate::schema::{users, categories, products, orders, campaigns, orders_products};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub title: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Category))]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub category_id: i32,
    pub author: String,
    pub list_price: f32,
    pub stock_quantity: i32,
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = campaigns)]
pub struct Campaign {
    pub id: i32,
    pub description: String,
    pub min_purchase_price: f32,
    pub min_purchase_quantity: i32,
    pub discount_quantity: i32,
    pub discount_percent: i32,
    pub rule_author: String,
    pub rule_category: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Campaign))]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub price_without_discount: f32,
    pub discounted_price: f32,
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
