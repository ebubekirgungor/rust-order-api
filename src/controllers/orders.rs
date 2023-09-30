use crate::controllers::functions;
use crate::insertables::NewOrder;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::models::{Order, Product};
use rust_order_api::schema;
use schema::orders::dsl::*;
use serde::Deserialize;
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

pub fn get_all_orders(conn: &mut PgConnection) -> Result<Vec<Order>, DbError> {
    let all_orders = orders.select(Order::as_select()).load(conn).expect("error");
    Ok(all_orders)
}

pub fn get_order_by_id(conn: &mut PgConnection, order_id: i32) -> Result<Order, DbError> {
    let order = orders
        .filter(id.eq(order_id))
        .first::<Order>(conn)
        .expect("error");
    Ok(order)
}

pub fn insert_new_order(
    conn: &mut PgConnection,
    _user_id: i32,
    _product_ids: Vec<i32>,
) -> Result<Order, DbError> {
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
        Some(min_campaign) => (
            min_campaign.discounted_price,
            min_campaign.campaign_id,
        ),
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
    for _product_id in _product_ids {
        diesel::insert_into(orders_products)
            .values((order_id.eq(created_order.id), product_id.eq(_product_id)))
            .execute(conn)?;
    }
    Ok(created_order)
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
    Ok(HttpResponse::Ok().json(all_orders))
}

#[get("/api/orders/{order_id}")]
async fn get_order(pool: web::Data<DbPool>, order_id: web::Path<i32>) -> Result<impl Responder> {
    let order = web::block(move || {
        let mut conn = pool.get()?;
        get_order_by_id(&mut conn, *order_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(order))
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
    Ok(HttpResponse::Created().json(order))
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
