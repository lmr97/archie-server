#!/bin/bash

# this script backs up the database and its data
# to db_schema.sql

backup() {
    echo "Backing up DB to ~/archie-server/db-init/db_schema.sql (on host)..."

    touch /docker-entrypoint-initdb.d/db_schema_save.sql
    mysqldump --protocol=tcp \
        --databases archie \
        --user server1 --password="$MYSQL_PASSWORD" \
        --skip-lock-tables \
        --result-file /docker-entrypoint-initdb.d/db_schema_save.sql
}

trap 'backup' SIGTERM

# default ENTRYPOINT for mysql image, that gets passed an argument
# of 'mysqld' as the CMD
# after backgrounding, the PID for the child == $!
docker-entrypoint.sh "${@}" &

# don't exit from this script until completion of the child process,
# or, more importantly, until it receives a signal
wait $!