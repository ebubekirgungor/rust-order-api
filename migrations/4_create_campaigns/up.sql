CREATE TABLE campaigns (
  id SERIAL PRIMARY KEY,
  description VARCHAR NOT NULL,
  min_purchase_price FLOAT,
  min_purchase_quantity INT,
  discount_quantity INT,
  discount_percent INT,
  rule_author VARCHAR,
  rule_category VARCHAR
);