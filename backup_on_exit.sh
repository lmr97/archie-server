#!/bin/bash

# this script backs up the database and its data
# to db_schema.sql

backup() {
    echo "Backing up DB to ~/archie-server/sql_scripts/db_schema.sql (on host)..."

    # this script runs with root privileges
    bash -c \
        'mysqldump --databases archie -uroot -p"$MYSQL_ROOT_PASSWORD" \
        -r /docker-entrypoint-initdb.d/db_schema.sql'
}

trap 'backup' SIGTERM

# default ENTRYPOINT for mysql image, that gets passed an argument
# of 'mysqld' as the CMD (which gets)
# after backgrounding, the PID for the child == $!
echo $1
docker-entrypoint.sh "${@}" &

# don't exit from this script until completion of the child process,
# or, more importantly, until it receives a signal
wait $!