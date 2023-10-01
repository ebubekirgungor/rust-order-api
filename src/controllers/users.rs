use crate::insertables::NewUser;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::models::{Order, User};
use rust_order_api::schema;
use schema::users::dsl::*;
use serde::Serialize;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Serialize)]
pub struct UserWithOrders {
    pub id: i32,
    pub username: String,
    pub orders: Vec<Order>,
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<User>, DbError> {
    let all_users = users.select(User::as_select()).load(conn).expect("error");
    Ok(all_users)
}

pub fn get_all_users_with_orders(conn: &mut PgConnection) -> Result<Vec<UserWithOrders>, DbError> {
    let all_users = users.select(User::as_select()).load(conn).expect("error");
    let orders = Order::belonging_to(&all_users)
        .select(Order::as_select())
        .load(conn)?;

    let users_with_orders: Vec<UserWithOrders> = all_users
        .iter()
        .map(|user| {
            let user_orders: Vec<Order> = orders
                .iter()
                .filter(|order| order.user_id == user.id)
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
