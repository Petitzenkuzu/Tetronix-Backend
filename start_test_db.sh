#!/bin/bash
set -e

docker compose up -d test-db

echo "Waiting for Postgres to be healthy..."
until docker exec tetronix-backend-test-db-1 pg_isready -U user -d test_db > /dev/null 2>&1; do
  sleep 2
done
echo "Postgres is ready."

docker exec -i tetronix-backend-test-db-1 psql -U user -d test_db < Schema.sql
echo "Schema initialized."
