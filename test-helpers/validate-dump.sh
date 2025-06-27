# the dump did not occur if the dump file is:
# - empty,
# - has an identical timestamp to the backup file, or
# - is missing any of the INSERT or CREATE TABLE statements for the tables

if [[ $(tail -n 3 db-init-demo/db_schema_save.sql) = $(tail -n 3 db-init-demo/db_schema.sql.bkup) ]]
then 
    exit 1
fi
cat db-init-demo/db_schema_save.sql | grep "CREATE TABLE \`guestbook\`"
cat db-init-demo/db_schema_save.sql | grep "INSERT INTO \`guestbook\`"
cat db-init-demo/db_schema_save.sql | grep "CREATE TABLE \`hitLog\`"
cat db-init-demo/db_schema_save.sql | grep "INSERT INTO \`hitLog\`"