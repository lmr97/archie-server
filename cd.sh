cd /home/martin/archie-server
docker compose down

# back up database locally
NOW_MINUTE=$(date +"%Y-%m-%dT%H:%I")
LATEST_DUMP="$(ls db-dumps | grep ${NOW_MINUTE})"
if [[ -n "$(cat db-dumps/$LATEST_DUMP)" ]] 
then
    mariadb -u server1 -p $MYSQL_PASSWORD --database=archie --execute "source db-dumps/${LATEST_DUMP}"
    cp db-dumps/$LATEST_DUMP db-init/db_schema.sql
else
    echo "Dump failed!"
    exit 1
fi


# update source
git pull --recurse-submodules

# update Docker images
docker pull lmr97/archie-svr:latest
docker pull lmr97/lb-web-app:latest

# run pod
docker compose up --detach