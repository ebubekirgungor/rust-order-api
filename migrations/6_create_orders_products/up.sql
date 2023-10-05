CREATE TABLE orders_products (
  order_id INT NOT NULL REFERENCES orders(id),
  product_id INT NOT NULL REFERENCES products(id),
  PRIMARY KEY(order_id, product_id)
);