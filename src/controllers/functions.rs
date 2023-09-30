use crate::orders;
use rust_order_api::models::Campaign;

pub fn get_available_campaigns(
    _campaigns: Vec<Campaign>,
    products: &mut Vec<orders::ProductWithCategory>,
) -> Vec<Campaign> {
    let mut available_campaigns = Vec::new();
    for campaign in _campaigns {
        let conditions = (campaign.rule_author.is_some()
            && campaign.rule_category.is_some()
            && campaign.min_purchase_quantity.is_some()
            && campaign.discount_quantity.is_some()
            && products
                .iter()
                .filter(|product| {
                    &product.product.author == campaign.rule_author.as_ref().unwrap()
                        && &product.category_title == campaign.rule_category.as_ref().unwrap()
                })
                .count() as i32
                >= campaign.min_purchase_quantity.unwrap())
            || (campaign.rule_author.is_some()
                && campaign.rule_category.is_some()
                && campaign.min_purchase_quantity.is_some()
                && campaign.discount_percent.is_some()
                && products
                    .iter()
                    .filter(|product| {
                        &product.product.author == campaign.rule_author.as_ref().unwrap()
                            && &product.category_title == campaign.rule_category.as_ref().unwrap()
                    })
                    .count() as i32
                    >= campaign.min_purchase_quantity.unwrap())
            || (campaign.rule_category.is_some()
                && campaign.rule_author.is_none()
                && campaign.min_purchase_quantity.is_some()
                && campaign.discount_quantity.is_some()
                && products
                    .iter()
                    .filter(|product| {
                        &product.category_title == campaign.rule_category.as_ref().unwrap()
                    })
                    .count() as i32
                    >= campaign.min_purchase_quantity.unwrap())
            || (campaign.rule_author.is_some()
                && campaign.rule_category.is_none()
                && campaign.min_purchase_quantity.is_some()
                && campaign.discount_quantity.is_some()
                && products
                    .iter()
                    .filter(|product| {
                        &product.product.author == campaign.rule_author.as_ref().unwrap()
                    })
                    .count() as i32
                    >= campaign.min_purchase_quantity.unwrap())
            || (campaign.rule_category.is_some()
                && campaign.rule_author.is_none()
                && campaign.min_purchase_quantity.is_some()
                && campaign.discount_percent.is_some()
                && products
                    .iter()
                    .filter(|product| {
                        &product.category_title == campaign.rule_category.as_ref().unwrap()
                    })
                    .count() as i32
                    >= campaign.min_purchase_quantity.unwrap())
            || (campaign.rule_author.is_some()
                && campaign.rule_category.is_none()
                && campaign.min_purchase_quantity.is_some()
                && campaign.discount_percent.is_some()
                && products
                    .iter()
                    .filter(|product| {
                        &product.product.author == campaign.rule_author.as_ref().unwrap()
                    })
                    .count() as i32
                    >= campaign.min_purchase_quantity.unwrap())
            || (campaign.min_purchase_quantity.is_some()
                && campaign.discount_quantity.is_some()
                && campaign.rule_category.is_none()
                && campaign.rule_author.is_none()
                && products.len() as i32 >= campaign.min_purchase_quantity.unwrap())
            || (campaign.min_purchase_quantity.is_some()
                && campaign.discount_percent.is_some()
                && campaign.rule_category.is_none()
                && campaign.rule_author.is_none()
                && products.len() as i32 >= campaign.min_purchase_quantity.unwrap())
            || (campaign.min_purchase_price.is_some()
                && products
                    .iter()
                    .map(|product| product.product.list_price)
                    .sum::<f64>()
                    >= campaign.min_purchase_price.unwrap());

        if conditions {
            if let Some(_discount_percent_value) = campaign.discount_percent {
                available_campaigns.push(Campaign {
                    id: campaign.id,
                    description: campaign.description,
                    min_purchase_price: campaign.min_purchase_price,
                    min_purchase_quantity: campaign.min_purchase_quantity,
                    discount_quantity: campaign.discount_quantity,
                    discount_percent: campaign.discount_percent,
                    rule_author: campaign.rule_author,
                    rule_category: campaign.rule_category,
                });
            } else if let Some(_discount_quantity_value) = campaign.discount_quantity {
                available_campaigns.push(Campaign {
                    id: campaign.id,
                    description: campaign.description,
                    min_purchase_price: campaign.min_purchase_price,
                    min_purchase_quantity: campaign.min_purchase_quantity,
                    discount_quantity: campaign.discount_quantity,
                    discount_percent: campaign.discount_percent,
                    rule_author: campaign.rule_author,
                    rule_category: campaign.rule_category,
                });
            }
        }
    }
    available_campaigns
}

pub fn get_discounted_total_price(
    campaign: &Campaign,
    products: &mut Vec<orders::ProductWithCategory>,
    total_price: f64,
) -> f64 {
    if let Some(discount_percent_value) = campaign.discount_percent {
        let discounted_price = total_price - (total_price * discount_percent_value as f64) / 100.0;
        discounted_price
    } else if let Some(discount_quantity_value) = campaign.discount_quantity {
        let mut eligible_products = products
            .iter()
            .filter(|product| {
                (campaign.rule_author.is_none()
                    || campaign.rule_author.as_ref() == Some(&product.product.author))
                    && (campaign.rule_category.is_none()
                        || campaign.rule_category.as_ref() == Some(&product.category_title))
            })
            .collect::<Vec<_>>();

        eligible_products.sort_by(|a, b| {
            a.product
                .list_price
                .partial_cmp(&b.product.list_price)
                .unwrap()
        });
        eligible_products.truncate(discount_quantity_value as usize);

        let discounted_price = total_price
            - eligible_products
                .iter()
                .map(|product| product.product.list_price)
                .sum::<f64>();

        discounted_price
    } else {
        0.0
    }
}
