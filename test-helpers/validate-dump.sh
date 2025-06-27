# the dump did not occur if the dump file is:
# - empty,
# - has an identical timestamp to the backup file, or
# - is missing any of the INSERT or CREATE TABLE statements for the tables

if [[ $(tail -n 3 db-dumps/db_schema_*.sql) = $(tail -n 3 db-init-demo/db_schema.sql.bkup) ]]
then 
    exit 1
fi
cat db-dumps/db_schema_*.sql | grep "CREATE TABLE \`guestbook\`"
cat db-dumps/db_schema_*.sql | grep "INSERT INTO \`guestbook\`"
cat db-dumps/db_schema_*.sql | grep "CREATE TABLE \`hitLog\`"
cat db-dumps/db_schema_*.sql | grep "INSERT INTO \`hitLog\`"