CREATE TABLE activation_code (
  email VARCHAR(100) PRIMARY KEY REFERENCES users(email) ON DELETE CASCADE,
  activation_code INTEGER NOT NULL,
  expiration_time TIMESTAMP NOT NULL DEFAULT now() + interval '30' minute,
  attempts SMALLINT DEFAULT 0
);
