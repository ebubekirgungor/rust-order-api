mod controllers {
    pub mod campaigns;
    pub mod products;
    pub mod users;
}
mod insertables;
use actix_web::{web, App, HttpServer};
use controllers::campaigns;
use controllers::products;
use controllers::users;
use diesel::{r2d2, PgConnection};
use dotenvy::dotenv;
use std::env;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = initialize_db_pool();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(users::get_users)
            .service(users::get_user)
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
    })
    .bind((
        "127.0.0.1",
        env::var("PORT")
            .expect("env_err")
            .parse::<u16>()
            .expect("parse_err"),
    ))?
    .run()
    .await
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("Variable not defined");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder().build(manager).expect("DB Error")
}
