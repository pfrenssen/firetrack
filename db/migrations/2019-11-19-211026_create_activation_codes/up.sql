CREATE TABLE activation_codes (
  id SERIAL PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  code INTEGER NOT NULL,
  expiration_time TIMESTAMP NOT NULL DEFAULT now() + interval '30' minute,
  attempts SMALLINT NOT NULL DEFAULT 0
);
