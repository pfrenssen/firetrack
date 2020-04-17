CREATE TABLE activation_codes (
  email VARCHAR(100) PRIMARY KEY REFERENCES users(email) ON DELETE CASCADE,
  code INTEGER NOT NULL,
  expiration_time TIMESTAMP NOT NULL DEFAULT now() + interval '30' minute,
  attempts SMALLINT NOT NULL DEFAULT 0
);
