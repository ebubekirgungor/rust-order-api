use diesel::insert_into;
use diesel::prelude::*;
use rust_order_api::establish_connection;
use rust_order_api::schema;
use schema::campaigns;
use schema::campaigns::dsl::*;
use schema::categories;
use schema::categories::dsl::*;
use schema::products;
use schema::products::dsl::*;
use schema::users;
use schema::users::dsl::*;
use serde::Deserialize;
use std::fs;
use std::io::Read;

#[derive(Deserialize, Insertable)]
struct Campaign {
    description: String,
    min_purchase_price: Option<f64>,
    min_purchase_quantity: Option<i32>,
    discount_quantity: Option<i32>,
    discount_percent: Option<i32>,
    rule_author: Option<String>,
    rule_category: Option<String>,
}

#[derive(Deserialize, Insertable)]
struct Categorie {
    title: String,
}

#[derive(Deserialize, Insertable)]
struct Product {
    title: String,
    category_id: i32,
    author: String,
    list_price: f64,
    stock_quantity: i32,
}

#[derive(Deserialize, Insertable)]
struct User {
    username: String,
}

fn main() -> std::io::Result<()> {
    let connection = &mut establish_connection();
    let mut campaigns_json = String::new();
    let mut categories_json = String::new();
    let mut products_json = String::new();
    let mut users_json = String::new();
    fs::File::open("src/bin/campaigns.json")
        .expect("can't open")
        .read_to_string(&mut campaigns_json)
        .unwrap();
    fs::File::open("src/bin/categories.json")
        .expect("can't open")
        .read_to_string(&mut categories_json)
        .unwrap();
    fs::File::open("src/bin/products.json")
        .expect("can't open")
        .read_to_string(&mut products_json)
        .unwrap();
    fs::File::open("src/bin/users.json")
        .expect("can't open")
        .read_to_string(&mut users_json)
        .unwrap();

    insert_into(campaigns)
        .values(serde_json::from_str::<Vec<Campaign>>(&campaigns_json).unwrap())
        .execute(connection)
        .unwrap();

    insert_into(categories)
        .values(serde_json::from_str::<Vec<Categorie>>(&categories_json).unwrap())
        .execute(connection)
        .unwrap();

    insert_into(products)
        .values(serde_json::from_str::<Vec<Product>>(&products_json).unwrap())
        .execute(connection)
        .unwrap();

    insert_into(users)
        .values(serde_json::from_str::<Vec<User>>(&users_json).unwrap())
        .execute(connection)
        .unwrap();
    Ok(())
}
