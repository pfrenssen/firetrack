CREATE TABLE categories (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  description VARCHAR(255),
  user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  parent_id INTEGER REFERENCES categories (id)
);
