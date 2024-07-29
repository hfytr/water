#!/usr/bin/env bash
get_default() {
    if [ -z "$1" ]; then
        echo "$2"
    else
        echo "$1"
    fi
}

export DB_USER=$(get_default "$POSTGRES_USER" "postgres")
export DB_PASSWORD=$(get_default "$POSTGRES_PASSWORD" "password")
export DB_NAME=$(get_default "$POSTGRES_DB" "db")
export DB_PORT=$(get_default "$POSTGRES_PORT" "5432")
export DB_HOST=$(get_default "$POSTGRES_HOST" "localhost")
export PGDATA=".pgdata"
export PGLOG="$PGDATA/postgresql.log"

if [ ! -d /run/postgresql ]; then
    sudo mkdir -p /run/postgresql
    sudo chown "$USER" /run/postgresql
fi

if [ ! -d "$PGDATA" ]; then
    echo "Initializing PostgreSQL database..."
    initdb --pgdata="$PGDATA"
fi

echo "listen_addresses = '*'" >> "$PGDATA/postgresql.conf"

echo "Starting PostgreSQL server on port 5432..."
pg_ctl -D "$PGDATA" -l "$PGLOG" -o "-p 5432 -c config_file=$PGDATA/postgresql.conf" start

psql -d postgres -c "CREATE ROLE postgres WITH LOGIN SUPERUSER CREATEDB CREATEROLE REPLICATION;"

export PGPASSWORD="$DB_PASSWORD"
until psql -h "localhost" -p "$DB_PORT" -U "$DB_USER" -d postgres -c '\q'; do
    echo "Postgres is still unavailable - sleeping" >&2
    sleep 1
done

echo "Postgres is up and running on port $DB_PORT!" >&2

export DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@localhost:$DB_PORT/$DB_NAME"
sqlx database create
if [ ! -d migrations ]; then
    mkdir migrations
fi
sqlx migrate run
echo "hi"

echo "Postgres has been migrated, ready to go!" >&2

on_exit() {
    pg_ctl --pgdata="$PGDATA" stop
}
trap on_exit EXIT
