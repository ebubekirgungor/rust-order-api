use crate::insertables::NewProduct;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::models::Product;
use rust_order_api::schema;
use schema::products::dsl::*;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub fn get_all_products(conn: &mut PgConnection) -> Result<Vec<Product>, DbError> {
    let all_products = products
        .select(Product::as_select())
        .load(conn)
        .expect("error");
    Ok(all_products)
}

pub fn get_product_by_id(conn: &mut PgConnection, product_id: i32) -> Result<Product, DbError> {
    let product = products
        .filter(id.eq(product_id))
        .first::<Product>(conn)
        .expect("error");
    Ok(product)
}

pub fn insert_new_product(
    conn: &mut PgConnection,
    _title: &str,
    _category_id: i32,
    _author: &str,
    _list_price: f64,
    _stock_quantity: i32,
) -> Result<NewProduct, DbError> {
    let new_product = NewProduct {
        title: _title.to_owned(),
        category_id: _category_id.to_owned(),
        author: _author.to_owned(),
        list_price: _list_price.to_owned(),
        stock_quantity: _stock_quantity.to_owned(),
    };
    diesel::insert_into(products)
        .values(&new_product)
        .execute(conn)?;
    Ok(new_product)
}

pub fn delete_product_by_id(conn: &mut PgConnection, product_id: i32) -> Result<String, DbError> {
    diesel::delete(products.filter(id.eq(product_id))).execute(conn)?;
    Ok("Product deleted".to_string())
}

#[get("/api/products")]
async fn get_products(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let all_products = web::block(move || {
        let mut conn = pool.get()?;
        get_all_products(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(all_products))
}

#[get("/api/products/{product_id}")]
async fn get_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<impl Responder> {
    let product = web::block(move || {
        let mut conn = pool.get()?;
        get_product_by_id(&mut conn, *product_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(product))
}

#[post("/api/products")]
async fn create_product(
    pool: web::Data<DbPool>,
    form: web::Json<NewProduct>,
) -> Result<impl Responder> {
    let product = web::block(move || {
        let mut conn = pool.get()?;
        insert_new_product(
            &mut conn,
            &form.title,
            form.category_id,
            &form.author,
            form.list_price,
            form.stock_quantity,
        )
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(product))
}

#[delete("/api/products/{product_id}")]
async fn delete_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<impl Responder> {
    let product = web::block(move || {
        let mut conn = pool.get()?;
        delete_product_by_id(&mut conn, *product_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(product))
}
