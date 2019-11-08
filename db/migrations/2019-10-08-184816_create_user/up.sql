CREATE TABLE users (
  email VARCHAR(100) NOT NULL PRIMARY KEY,
  password VARCHAR(120) NOT NULL,
  created TIMESTAMP NOT NULL,
  validated BOOLEAN NOT NULL DEFAULT 'f'
);