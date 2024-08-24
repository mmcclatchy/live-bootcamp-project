#!/bin/bash
set -e

# Start PostgreSQL
docker-entrypoint.sh postgres &

# Wait for PostgreSQL to be ready
until pg_isready -h localhost -p 5432 -U postgres
do
  echo "Waiting for postgres..."
  sleep 2;
done
echo "PostgreSQL started"

# Run your cleanup script
psql -U postgres -f /docker-entrypoint-initdb.d/test_db_cleanup.sql

# Keep the container running
tail -f /dev/null
