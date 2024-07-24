#!/usr/bin/env fish
set -x
set -eo pipefail

function get_default
    if test "$argv[1]" = ""
        echo $argv[2]
    else
        echo $argv[1]
    end
end

set DB_USER (get_default $POSTGRES_USER 'postgres')
set DB_PASSWORD (get_default $POSTGRES_PASSWORD 'password')
set DB_NAME (get_default $POSTGRES_DB 'newsletter')
set DB_PORT (get_default $POSTGRES_PORT '5432')

if not test -d /run/postgresql
    sudo mkdir -p /run/postgresql
    sudo chown $USER /run/postgresql
end

if not test -d $PGDATA
    echo "Initializing PostgreSQL database..."
    initdb --pgdata=$PGDATA
end

echo "listen_addresses = '*'" >> $PGDATA/postgresql.conf

echo "Starting PostgreSQL server on port 5432..."
pg_ctl -D $PGDATA -l $PGLOG -o "-p 5432" start

function on_exit
    pg_ctl -D $PGDATA stop
end
trap on_exit EXIT

set -x PGPASSWORD $DB_PASSWORD
while not psql -h "localhost" -p "$DB_PORT" -U "$DB_USER" -d postgres -c '\q'
    echo "Postgres is still unavailable - sleeping" >&2
    sleep 1
end

echo "Postgres is up and running on port $DB_PORT!" >&2

set -x DATABASE_URL "postgres://$DB_USER:$DB_PASSWORD@localhost:$DB_PORT/$DB_NAME"
sqlx database create
sqlx migrate run

createuser --superuser postgres

echo "Postgres has been migrated, ready to go!" >&2
