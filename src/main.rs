mod controllers {
    pub mod campaigns;
    pub mod functions;
    pub mod orders;
    pub mod products;
    pub mod users;
}
mod insertables;
use actix_web::{web, App, HttpServer};
use apalis::prelude::*;
use apalis::{layers::TraceLayer, redis::RedisStorage};
use controllers::campaigns;
use controllers::orders;
use controllers::products;
use controllers::users;
use diesel::{r2d2, PgConnection};
use dotenvy::dotenv;
use futures::future;
use serde::{Deserialize, Serialize};
use std::env;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
use r2d2_redis::{r2d2 as redis_r2d2, RedisConnectionManager};
type RedisPool = redis_r2d2::Pool<RedisConnectionManager>;

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryOrder {
    pub id: i32,
    pub price_without_discount: f64,
    pub discounted_price: f64,
    pub campaign_id: Option<i32>,
    pub user_id: i32,
}

impl Job for QueryOrder {
    const NAME: &'static str = "apalis::QueryOrder";
}

async fn order_service(_job: QueryOrder, _ctx: JobContext) {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = initialize_db_pool();
    let redis_pool = initialize_redis_pool();
    let storage = RedisStorage::connect("redis://127.0.0.1/")
        .await
        .expect("err");
    let storage_data = web::Data::new(storage.clone());
    let http = async {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(db_pool.clone()))
                .app_data(web::Data::new(redis_pool.clone()))
                .app_data(storage_data.clone())
                .service(users::get_users)
                .service(users::get_users_with_orders)
                .service(users::get_user)
                .service(users::get_user_with_orders)
                .service(users::create_user)
                .service(users::delete_user)
                .service(products::get_products)
                .service(products::get_product)
                .service(products::create_product)
                .service(products::delete_product)
                .service(campaigns::get_campaigns)
                .service(campaigns::get_campaign)
                .service(campaigns::create_campaign)
                .service(campaigns::delete_campaign)
                .service(orders::get_orders)
                .service(orders::get_order)
                .service(orders::create_order)
                .service(orders::delete_order)
        })
        .bind((
            "127.0.0.1",
            env::var("PORT")
                .expect("env_err")
                .parse::<u16>()
                .expect("parse_err"),
        ))?
        .run()
        .await?;
        Ok(())
    };
    let worker = Monitor::new()
        .register_with_count(2, move |index| {
            WorkerBuilder::new(format!("order-queue-{index}"))
                .layer(TraceLayer::new())
                .with_storage(storage.clone())
                .build_fn(order_service)
        })
        .run();

    future::try_join(http, worker).await?;
    Ok(())
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("Variable not defined");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder().build(manager).expect("DB Error")
}

fn initialize_redis_pool() -> RedisPool {
    let manager = RedisConnectionManager::new("redis://127.0.0.1:6379").unwrap();
    redis_r2d2::Pool::builder()
        .build(manager)
        .expect("Redis Error")
}
