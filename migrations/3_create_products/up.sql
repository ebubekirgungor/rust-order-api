CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  category_id INT NOT NULL REFERENCES categories(id),
  author VARCHAR NOT NULL,
  list_price FLOAT NOT NULL,
  stock_quantity INT NOT NULL
);