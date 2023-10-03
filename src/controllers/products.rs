use crate::insertables::NewProduct;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::schema;
use schema::products::dsl::*;
use serde::Serialize;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Debug, Queryable, Serialize)]
pub struct ProductWithCategory {
    pub id: i32,
    pub title: String,
    pub category_id: i32,
    pub author: String,
    pub list_price: f64,
    pub stock_quantity: i32,
    pub category: CategoryTitle,
}

#[derive(Debug, Queryable, Serialize)]
pub struct CategoryTitle {
    pub title: String,
}

pub fn get_all_products(conn: &mut PgConnection) -> Result<Vec<ProductWithCategory>, DbError> {
    use schema::categories::dsl::*;
    let all_products = products
        .inner_join(categories.on(schema::products::category_id.eq(schema::categories::id)))
        .select((
            schema::products::id,
            schema::products::title,
            schema::products::category_id,
            schema::products::author,
            schema::products::list_price,
            schema::products::stock_quantity,
            schema::categories::title,
        ))
        .load::<(i32, String, i32, String, f64, i32, String)>(conn)?
        .into_iter()
        .map(
            |(
                product_id,
                product_title,
                product_category_id,
                product_author,
                product_list_price,
                product_stock_quantity,
                category_info,
            )| {
                let category = CategoryTitle {
                    title: category_info,
                };

                ProductWithCategory {
                    id: product_id,
                    title: product_title,
                    category_id: product_category_id,
                    author: product_author,
                    list_price: product_list_price,
                    stock_quantity: product_stock_quantity,
                    category,
                }
            },
        )
        .collect();

    Ok(all_products)
}

pub fn get_product_by_id(
    conn: &mut PgConnection,
    product_id: i32,
) -> Result<ProductWithCategory, DbError> {
    use schema::categories::dsl::*;
    let product_with_category = products
        .filter(schema::products::id.eq(product_id))
        .inner_join(categories.on(schema::products::category_id.eq(schema::categories::id)))
        .select((
            schema::products::id,
            schema::products::title,
            schema::products::category_id,
            schema::products::author,
            schema::products::list_price,
            schema::products::stock_quantity,
            schema::categories::title,
        ))
        .first::<(i32, String, i32, String, f64, i32, String)>(conn)?;

    let (
        product_id,
        product_title,
        product_category_id,
        product_author,
        product_list_price,
        product_stock_quantity,
        category_info,
    ) = product_with_category;
    let category = CategoryTitle {
        title: category_info,
    };

    Ok(ProductWithCategory {
        id: product_id,
        title: product_title,
        category_id: product_category_id,
        author: product_author,
        list_price: product_list_price,
        stock_quantity: product_stock_quantity,
        category,
    })
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

    let all_products_json = serde_json::to_string_pretty(&all_products).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(all_products_json))
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

    let product_json = serde_json::to_string_pretty(&product).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(product_json))
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

    let product_json = serde_json::to_string_pretty(&product).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(product_json))
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
