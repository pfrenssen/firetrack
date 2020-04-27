CREATE TABLE expenses (
  id SERIAL PRIMARY KEY,
  amount NUMERIC(9, 2) NOT NULL,
  description VARCHAR(255),
  category_id INTEGER REFERENCES categories (id) NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  date TIMESTAMP NOT NULL
);
