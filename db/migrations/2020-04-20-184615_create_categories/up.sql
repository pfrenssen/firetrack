CREATE TABLE categories (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  description VARCHAR(255),
  user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  parent_id INTEGER REFERENCES categories (id)
);

CREATE UNIQUE INDEX categories_unique_rootcat_index ON categories (name, user_id) WHERE parent_id IS NULL;

CREATE UNIQUE INDEX categories_unique_subcat_index ON categories (name, user_id, parent_id) WHERE parent_id IS NOT NULL;
