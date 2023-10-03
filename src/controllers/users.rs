use crate::insertables::NewUser;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::models::{Order, OrderToProduct, Product, User};
use rust_order_api::schema;
use schema::users::dsl::*;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Serialize)]
pub struct UserWithOrders {
    pub id: i32,
    pub username: String,
    pub orders: Vec<Value>,
}

#[derive(Queryable, Serialize, Debug, Clone)]
pub struct OrderWithFields {
    id: i32,
    price_without_discount: f64,
    discounted_price: f64,
    campaign_id: Option<i32>,
    user_id: i32,
    campaign_description: Option<String>,
}

#[derive(Queryable, Debug)]
pub struct ProductWithCategory {
    pub product: Product,
    pub category_title: String,
}

fn get_all_orders(conn: &mut PgConnection) -> Result<Vec<Value>, DbError> {
    use schema::campaigns::dsl::*;
    use schema::categories::dsl::*;
    use schema::orders::dsl::*;
    use schema::products::dsl::*;

    let all_orders: HashMap<i32, Order> = orders
        .select((schema::orders::id, Order::as_select()))
        .load::<(i32, Order)>(conn)?
        .into_iter()
        .collect();

    let order_values: Vec<&Order> = all_orders.values().collect();

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
        orders_json.push(json!({
            "id": order.id,
            "price_without_discount": order.price_without_discount,
            "discounted_price": order.discounted_price,
            "campaign_id": order.campaign_id,
            "user_id": order.user_id,
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
    Ok(orders_json)
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<User>, DbError> {
    let all_users = users.select(User::as_select()).load(conn).expect("error");
    Ok(all_users)
}

pub fn get_all_users_with_orders(conn: &mut PgConnection) -> Result<Vec<UserWithOrders>, DbError> {
    let all_users = users.select(User::as_select()).load(conn).expect("error");

    let users_with_orders: Vec<UserWithOrders> = all_users
        .iter()
        .map(|user| {
            let user_orders = get_all_orders(conn)
                .unwrap()
                .iter()
                .filter(|order| order.get("user_id") == Some(&Value::from(user.id)))
                .cloned()
                .collect();
            UserWithOrders {
                id: user.id,
                username: user.username.clone(),
                orders: user_orders,
            }
        })
        .collect();
    Ok(users_with_orders)
}

pub fn get_user_by_id(conn: &mut PgConnection, user_id: i32) -> Result<User, DbError> {
    let user = users
        .filter(id.eq(user_id))
        .first::<User>(conn)
        .expect("error");
    Ok(user)
}

pub fn get_user_by_id_with_orders(
    conn: &mut PgConnection,
    user_id: i32,
) -> Result<UserWithOrders, DbError> {
    let user = users
        .filter(id.eq(user_id))
        .first::<User>(conn)
        .expect("error");

    let user_orders = get_all_orders(conn)
        .unwrap()
        .iter()
        .filter(|order| order.get("user_id") == Some(&Value::from(user.id)))
        .cloned()
        .collect();

    let user_with_orders = UserWithOrders {
        id: user.id,
        username: user.username.clone(),
        orders: user_orders,
    };
    Ok(user_with_orders)
}

pub fn insert_new_user(conn: &mut PgConnection, _username: &str) -> Result<NewUser, DbError> {
    let new_user = NewUser {
        username: _username.to_owned(),
    };
    diesel::insert_into(users).values(&new_user).execute(conn)?;
    Ok(new_user)
}

pub fn delete_user_by_id(conn: &mut PgConnection, user_id: i32) -> Result<String, DbError> {
    diesel::delete(users.filter(id.eq(user_id))).execute(conn)?;
    Ok("User deleted".to_string())
}

#[get("/api/users")]
async fn get_users(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let all_users = web::block(move || {
        let mut conn = pool.get()?;
        get_all_users(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(all_users))
}

#[get("/api/users_with_orders")]
async fn get_users_with_orders(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let all_users = web::block(move || {
        let mut conn = pool.get()?;
        get_all_users_with_orders(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(all_users))
}

#[get("/api/users/{user_id}")]
async fn get_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        get_user_by_id(&mut conn, *user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/api/users_with_orders/{user_id}")]
async fn get_user_with_orders(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
) -> Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        get_user_by_id_with_orders(&mut conn, *user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/api/users")]
async fn create_user(pool: web::Data<DbPool>, form: web::Json<NewUser>) -> Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        insert_new_user(&mut conn, &form.username)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(user))
}

#[delete("/api/users/{user_id}")]
async fn delete_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        delete_user_by_id(&mut conn, *user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(user))
}
