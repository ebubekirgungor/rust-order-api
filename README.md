# Book order api written in Rust & Actix & Diesel ORM

## Installation

- Clone repository

```bash
$ git clone https://github.com/ebubekirgungor/rust-order-api.git
```

- Change environment variables in .env file
- Run

```bash
# Install diesel client
$ cargo install diesel_cli
# Run PostgreSql and Redis using docker
$ docker compose up -d
# Migrate database
$ diesel migration run
# Seed database
$ cargo run --bin seed
```

## Running the app

```bash
$ cargo run --bin rust-order-api
```

## Using

- Create order

```
POST /api/orders
# Example
{
    "user_id": 1,
    "product_ids": [1, 2, 3]
}
```

- Get an order by id

```
GET /api/orders/{id}
```

- Get all orders

```
GET /api/orders
```

- Get all campaigns

```
GET /api/campaigns
```

- Get all users

```
GET /api/users
```

- Get all users including orders

```
GET /api/users_with_orders
```

- Get a user by id

```
GET /api/users/{id}
```

- Get a user by id including orders

```
GET /api/users_with_orders/{id}
```
