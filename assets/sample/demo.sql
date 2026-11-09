PRAGMA foreign_keys = ON;

CREATE TABLE customers (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    country     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id           INTEGER PRIMARY KEY,
    customer_id  INTEGER NOT NULL REFERENCES customers(id),
    order_total  REAL NOT NULL CHECK (order_total >= 0),
    status       TEXT NOT NULL DEFAULT 'pending',
    ordered_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_status ON orders(status);

CREATE VIEW customer_order_totals AS
SELECT c.id AS customer_id, c.name, COUNT(o.id) AS order_count,
       COALESCE(SUM(o.order_total), 0) AS lifetime_value
FROM customers AS c
LEFT JOIN orders AS o ON o.customer_id = c.id
GROUP BY c.id, c.name;

INSERT INTO customers (name, email, country) VALUES
    ('Avery Stone', 'avery@example.test', 'United States'),
    ('Mila Novak', 'mila@example.test', 'Ukraine'),
    ('Noah Fischer', 'noah@example.test', 'Germany'),
    ('Sofia Costa', 'sofia@example.test', 'Portugal');

INSERT INTO orders (customer_id, order_total, status) VALUES
    (1, 129.90, 'paid'),
    (1, 49.00, 'shipped'),
    (2, 310.50, 'paid'),
    (3, 82.25, 'pending'),
    (4, 199.99, 'cancelled');
