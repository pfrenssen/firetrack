#!/bin/bash

# Generate comma-separated list of 32 integers in the range of 0-255
session_key=$(shuf -i 0-255 -n 32 | tr '\n' ',' | sed 's/,$//')

# Generate random string of 32 characters for secret key
secret_key=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 32)

# Write content to .env file
cat << EOF > .env
SESSION_KEY=$session_key
SECRET_KEY=$secret_key
EOF

echo "File created: .env"

