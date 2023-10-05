CREATE TABLE orders (
  id SERIAL PRIMARY KEY,
  price_without_discount FLOAT NOT NULL,
  discounted_price FLOAT NOT NULL,
  campaign_id INT REFERENCES campaigns(id),
  user_id INT NOT NULL REFERENCES users(id)
);