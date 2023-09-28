// @generated automatically by Diesel CLI.

diesel::table! {
    campaigns (id) {
        id -> Int4,
        description -> Varchar,
        min_purchase_price -> Nullable<Float8>,
        min_purchase_quantity -> Nullable<Int4>,
        discount_quantity -> Nullable<Int4>,
        discount_percent -> Nullable<Int4>,
        rule_author -> Nullable<Varchar>,
        rule_category -> Nullable<Varchar>,
    }
}

diesel::table! {
    categories (id) {
        id -> Int4,
        title -> Varchar,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        price_without_discount -> Float8,
        discounted_price -> Float8,
        campaign_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    orders_products (order_id, product_id) {
        order_id -> Int4,
        product_id -> Int4,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        title -> Varchar,
        category_id -> Int4,
        author -> Varchar,
        list_price -> Float8,
        stock_quantity -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
    }
}

diesel::joinable!(orders -> campaigns (campaign_id));
diesel::joinable!(orders -> users (user_id));
diesel::joinable!(orders_products -> orders (order_id));
diesel::joinable!(orders_products -> products (product_id));
diesel::joinable!(products -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    campaigns,
    categories,
    orders,
    orders_products,
    products,
    users,
);
