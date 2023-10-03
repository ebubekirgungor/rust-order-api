use crate::controllers::functions;
use crate::insertables::NewOrder;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::dsl::sql;
use diesel::sql_types::Text;
use diesel::{prelude::*, r2d2};
use rust_order_api::models::{Order, Product};
use rust_order_api::schema::{self};
use schema::orders::dsl::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Deserialize)]
struct OrderDto {
    user_id: i32,
    product_ids: Vec<i32>,
}

#[derive(Queryable, Debug)]
pub struct ProductWithCategory {
    pub product: Product,
    pub category_title: String,
}

#[derive(Queryable, Debug)]
pub struct OrderWithFields {
    id: i32,
    price_without_discount: f64,
    discounted_price: f64,
    campaign_id: Option<i32>,
    user_id: i32,
    username: String,
    campaign_description: Option<String>,
}

pub fn get_all_orders(conn: &mut PgConnection) -> Result<Vec<Value>, DbError> {
    use rust_order_api::models::OrderToProduct;
    use schema::campaigns::dsl::*;
    use schema::categories::dsl::*;
    use schema::products::dsl::*;
    use schema::users::dsl::*;

    let all_orders: HashMap<i32, Order> = orders
        .select((schema::orders::id, Order::as_select()))
        .load::<(i32, Order)>(conn)?
        .into_iter()
        .collect();

    let order_values: Vec<&Order> = all_orders.values().collect();
    let user_ids: Vec<i32> = all_orders.values().map(|order| order.user_id).collect();

    let usernames: HashMap<i32, String> = users
        .filter(schema::users::dsl::id.eq_any(user_ids))
        .select((schema::users::id, sql::<Text>("username")))
        .load::<(i32, String)>(conn)?
        .into_iter()
        .collect();

    let order_with_fields: Vec<OrderWithFields> = orders
        .inner_join(users.on(schema::orders::dsl::user_id.eq(schema::users::id)))
        .left_outer_join(
            campaigns.on(schema::orders::dsl::campaign_id.eq(schema::campaigns::id.nullable())),
        )
        .select((
            schema::orders::id,
            schema::orders::price_without_discount,
            schema::orders::discounted_price,
            schema::orders::campaign_id.nullable(),
            schema::orders::user_id,
            schema::users::username,
            schema::campaigns::description.nullable(),
        ))
        .load(conn)
        .expect("err");

    let all_products: HashMap<i32, Vec<ProductWithCategory>> =
        OrderToProduct::belonging_to(&order_values)
            .inner_join(products.inner_join(categories))
            .select((
                schema::orders_products::order_id,
                Product::as_select(),
                schema::categories::title,
            ))
            .load::<(i32, Product, String)>(conn)?
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc, (order_id, product, category_title)| {
                    acc.entry(order_id)
                        .or_insert_with(Vec::new)
                        .push(ProductWithCategory {
                            product,
                            category_title,
                        });
                    acc
                },
            );

    let mut orders_json = vec![];

    for order in order_with_fields {
        let default_products: Vec<ProductWithCategory> = vec![];
        let products_for_order = all_products.get(&order.id).unwrap_or(&default_products);
        if let Some(_username) = usernames.get(&order.user_id) {
            orders_json.push(json!({
                "id": order.id,
                "price_without_discount": order.price_without_discount,
                "discounted_price": order.discounted_price,
                "campaign_id": order.campaign_id,
                "user_id": order.user_id,
                "user": {
                    "username": _username,
                },
                "campaign": match order.campaign_description {
                    Some(campaign_description) => {
                        json!({
                            "description": campaign_description,
                        })
                    }
                    None => json!(null),
                },
                "products": products_for_order.iter().map(|product| {
                    json!({
                        "id": product.product.id,
                        "title": product.product.title,
                        "author": product.product.author,
                        "list_price": product.product.list_price,
                        "stock_quantity": product.product.stock_quantity,
                        "category": {
                            "title": product.category_title,
                        },
                    })
                }).collect::<Vec<_>>()
            }));
        }
    }
    Ok(orders_json)
}

pub fn get_order_by_id(conn: &mut PgConnection, order_id: i32) -> Result<Value, DbError> {
    use rust_order_api::models::OrderToProduct;
    use schema::campaigns::dsl::*;
    use schema::categories::dsl::*;
    use schema::products::dsl::*;
    use schema::users::dsl::*;

    let order = orders
        .filter(schema::orders::dsl::id.eq(order_id))
        .first::<Order>(conn)
        .expect("err");

    let order_with_fields: OrderWithFields = orders
        .filter(schema::orders::dsl::id.eq(order_id))
        .inner_join(users.on(schema::orders::dsl::user_id.eq(schema::users::id)))
        .left_outer_join(
            campaigns.on(schema::orders::dsl::campaign_id.eq(schema::campaigns::id.nullable())),
        )
        .select((
            schema::orders::id,
            schema::orders::price_without_discount,
            schema::orders::discounted_price,
            schema::orders::campaign_id.nullable(),
            schema::orders::user_id,
            schema::users::username,
            schema::campaigns::description.nullable(),
        ))
        .first(conn)
        .expect("err");

    let all_products = OrderToProduct::belonging_to(&order)
        .inner_join(products.inner_join(categories))
        .select((Product::as_select(), schema::categories::title))
        .load::<ProductWithCategory>(conn)?;

    let order_json = json!({
        "id": order_with_fields.id,
        "price_without_discount": order_with_fields.price_without_discount,
        "discounted_price": order_with_fields.discounted_price,
        "campaign_id": order_with_fields.campaign_id,
        "user_id": order_with_fields.user_id,
        "user": {
            "username": order_with_fields.username,
        },
        "campaign": match order_with_fields.campaign_description {
            Some(campaign_description) => {
                json!({
                    "description": campaign_description,
                })
            }
            None => json!(null),
        },
        "products": all_products.iter().map(|product| {
            json!({
                "id": product.product.id,
                "title": product.product.title,
                "author": product.product.author,
                "list_price": product.product.list_price,
                "stock_quantity": product.product.stock_quantity,
                "category": {
                    "title": product.category_title,
                },
            })
        }).collect::<Vec<_>>(),
    });

    Ok(order_json)
}

pub fn insert_new_order(
    conn: &mut PgConnection,
    _user_id: i32,
    _product_ids: Vec<i32>,
) -> Result<Value, DbError> {
    let shipping_cost = 35.0;
    use rust_order_api::models::Campaign;
    use rust_order_api::models::User;
    use schema::campaigns::dsl::*;
    use schema::categories::dsl::*;
    use schema::orders_products::dsl::*;
    use schema::products::dsl::*;
    use schema::users::dsl::*;

    users
        .filter(schema::users::dsl::id.eq(_user_id))
        .first::<User>(conn)
        .expect("error");

    diesel::update(products)
        .filter(schema::products::dsl::id.eq_any(&_product_ids))
        .set(stock_quantity.eq(stock_quantity - 1))
        .execute(conn)
        .expect("error");

    let mut order_products = products
        .filter(schema::products::dsl::id.eq_any(&_product_ids))
        .inner_join(categories)
        .select((Product::as_select(), schema::categories::title))
        .load::<ProductWithCategory>(conn)
        .expect("error");

    let all_campaigns = campaigns
        .select(Campaign::as_select())
        .load(conn)
        .expect("error");

    let mut total_price: f64 = order_products
        .iter()
        .map(|product| product.product.list_price)
        .sum();

    if total_price < 150.0 {
        total_price += shipping_cost;
    }

    let available_campaigns =
        functions::get_available_campaigns(all_campaigns, &mut order_products);

    #[derive(Debug)]
    struct DiscountedPrices {
        campaign_id: Option<i32>,
        discounted_price: f64,
    }
    let mut discounted_prices: Vec<DiscountedPrices> = vec![];

    for campaign in available_campaigns {
        let _discounted_price =
            functions::get_discounted_total_price(&campaign, &mut order_products, total_price);
        discounted_prices.push(DiscountedPrices {
            campaign_id: Some(campaign.id),
            discounted_price: _discounted_price,
        });
    }

    let min_discounted_campaign = discounted_prices
        .iter()
        .min_by(|a, b| a.discounted_price.partial_cmp(&b.discounted_price).unwrap());

    let (_discounted_price, _campaign_id) = match min_discounted_campaign {
        Some(min_campaign) => (min_campaign.discounted_price, min_campaign.campaign_id),
        None => (total_price, None),
    };

    let new_order = NewOrder {
        price_without_discount: (total_price * 1000.0).round() / 1000.0,
        discounted_price: (_discounted_price * 1000.0).round() / 1000.0,
        campaign_id: _campaign_id,
        user_id: _user_id.to_owned(),
    };

    let created_order: Order = diesel::insert_into(orders)
        .values(&new_order)
        .get_result(conn)?;

    let order_with_fields: OrderWithFields = orders
        .filter(schema::orders::dsl::id.eq(created_order.id))
        .inner_join(users.on(schema::orders::dsl::user_id.eq(schema::users::id)))
        .left_outer_join(
            campaigns.on(schema::orders::dsl::campaign_id.eq(schema::campaigns::id.nullable())),
        )
        .select((
            schema::orders::id,
            schema::orders::price_without_discount,
            schema::orders::discounted_price,
            schema::orders::campaign_id.nullable(),
            schema::orders::user_id,
            schema::users::username,
            schema::campaigns::description.nullable(),
        ))
        .first(conn)?;

    let order_json = json!({
        "id": order_with_fields.id,
        "price_without_discount": order_with_fields.price_without_discount,
        "discounted_price": order_with_fields.discounted_price,
        "campaign_id": order_with_fields.campaign_id,
        "user_id": order_with_fields.user_id,
        "user": {
            "username": order_with_fields.username,
        },
        "campaign": match order_with_fields.campaign_description {
            Some(campaign_description) => {
                json!({
                    "description": campaign_description,
                })
            }
            None => json!(null),
        },
        "products": order_products.iter().map(|product| {
            json!({
                "id": product.product.id,
                "title": product.product.title,
                "author": product.product.author,
                "list_price": product.product.list_price,
                "stock_quantity": product.product.stock_quantity,
                "category": {
                    "title": product.category_title,
                },
            })
        }).collect::<Vec<_>>(),
    });

    for _product_id in _product_ids {
        diesel::insert_into(orders_products)
            .values((order_id.eq(created_order.id), product_id.eq(_product_id)))
            .execute(conn)?;
    }
    Ok(order_json)
}

pub fn delete_order_by_id(conn: &mut PgConnection, order_id: i32) -> Result<String, DbError> {
    diesel::delete(orders.filter(id.eq(order_id))).execute(conn)?;
    Ok("Order deleted".to_string())
}

#[get("/api/orders")]
async fn get_orders(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let all_orders = web::block(move || {
        let mut conn = pool.get()?;
        get_all_orders(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let all_orders_json = serde_json::to_string_pretty(&all_orders).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(all_orders_json))
}

#[get("/api/orders/{order_id}")]
async fn get_order(pool: web::Data<DbPool>, order_id: web::Path<i32>) -> Result<impl Responder> {
    let order = web::block(move || {
        let mut conn = pool.get()?;
        get_order_by_id(&mut conn, *order_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let order_json = serde_json::to_string_pretty(&order).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(order_json))
}

#[post("/api/orders")]
async fn create_order(
    pool: web::Data<DbPool>,
    form: web::Json<OrderDto>,
) -> Result<impl Responder> {
    let order = web::block(move || {
        let mut conn = pool.get()?;
        let product_ids = form.product_ids.clone();
        insert_new_order(&mut conn, form.user_id, product_ids)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let order_json = serde_json::to_string_pretty(&order).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(order_json))
}

#[delete("/api/orders/{order_id}")]
async fn delete_order(pool: web::Data<DbPool>, order_id: web::Path<i32>) -> Result<impl Responder> {
    let order = web::block(move || {
        let mut conn = pool.get()?;
        delete_order_by_id(&mut conn, *order_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(order))
}
