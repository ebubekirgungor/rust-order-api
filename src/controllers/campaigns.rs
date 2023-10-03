use crate::insertables::NewCampaign;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder, Result};
use diesel::{prelude::*, r2d2};
use rust_order_api::models::Campaign;
use rust_order_api::schema;
use schema::campaigns::dsl::*;
type DbError = Box<dyn std::error::Error + Send + Sync>;
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub fn get_all_campaigns(conn: &mut PgConnection) -> Result<Vec<Campaign>, DbError> {
    let all_campaigns = campaigns
        .select(Campaign::as_select())
        .load(conn)
        .expect("error");
    Ok(all_campaigns)
}

pub fn get_campaign_by_id(conn: &mut PgConnection, campaign_id: i32) -> Result<Campaign, DbError> {
    let campaign = campaigns
        .filter(id.eq(campaign_id))
        .first::<Campaign>(conn)
        .expect("error");
    Ok(campaign)
}

pub fn insert_new_campaign(
    conn: &mut PgConnection,
    _description: &str,
    _min_purchase_price: f64,
    _min_purchase_quantity: i32,
    _discount_quantity: i32,
    _discount_percent: i32,
    _rule_author: &str,
    _rule_category: &str,
) -> Result<NewCampaign, DbError> {
    let new_campaign = NewCampaign {
        description: _description.to_owned(),
        min_purchase_price: _min_purchase_price.to_owned(),
        min_purchase_quantity: _min_purchase_quantity.to_owned(),
        discount_quantity: _discount_quantity.to_owned(),
        discount_percent: _discount_percent.to_owned(),
        rule_author: _rule_author.to_owned(),
        rule_category: _rule_category.to_owned(),
    };
    diesel::insert_into(campaigns)
        .values(&new_campaign)
        .execute(conn)?;
    Ok(new_campaign)
}

pub fn delete_campaign_by_id(conn: &mut PgConnection, campaign_id: i32) -> Result<String, DbError> {
    diesel::delete(campaigns.filter(id.eq(campaign_id))).execute(conn)?;
    Ok("Campaign deleted".to_string())
}

#[get("/api/campaigns")]
async fn get_campaigns(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let all_campaigns = web::block(move || {
        let mut conn = pool.get()?;
        get_all_campaigns(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let all_campaigns_json = serde_json::to_string_pretty(&all_campaigns).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(all_campaigns_json))
}

#[get("/api/campaigns/{campaign_id}")]
async fn get_campaign(
    pool: web::Data<DbPool>,
    campaign_id: web::Path<i32>,
) -> Result<impl Responder> {
    let campaign = web::block(move || {
        let mut conn = pool.get()?;
        get_campaign_by_id(&mut conn, *campaign_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let campaign_json = serde_json::to_string_pretty(&campaign).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(campaign_json))
}

#[post("/api/campaigns")]
async fn create_campaign(
    pool: web::Data<DbPool>,
    form: web::Json<NewCampaign>,
) -> Result<impl Responder> {
    let campaign = web::block(move || {
        let mut conn = pool.get()?;
        insert_new_campaign(
            &mut conn,
            &form.description,
            form.min_purchase_price,
            form.min_purchase_quantity,
            form.discount_quantity,
            form.discount_percent,
            &form.rule_author,
            &form.rule_category,
        )
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let campaign_json = serde_json::to_string_pretty(&campaign).expect("err");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(campaign_json))
}

#[delete("/api/campaigns/{campaign_id}")]
async fn delete_campaign(
    pool: web::Data<DbPool>,
    campaign_id: web::Path<i32>,
) -> Result<impl Responder> {
    let campaign = web::block(move || {
        let mut conn = pool.get()?;
        delete_campaign_by_id(&mut conn, *campaign_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(campaign))
}
