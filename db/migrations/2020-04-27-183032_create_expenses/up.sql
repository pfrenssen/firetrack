CREATE TABLE expenses (
  id SERIAL PRIMARY KEY,
  amount NUMERIC(9, 2) NOT NULL,
  category_id INTEGER REFERENCES categories (id) NOT NULL,
  date TIMESTAMP NOT NULL
);
