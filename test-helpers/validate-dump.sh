# the dump did not occur if the dump file is:
# - empty,
# - has an identical timestamp to the backup file, or
# - is missing any of the INSERT or CREATE TABLE statements for the tables

if [  $(tail -n 1 db-init-demo/db_schema.sql) \
    = $(tail -n 1 db-init-demo/db_schema.sql.bkup) ] 
then 
    exit 1
fi
cat db-init-demo/db_schema.sql | grep "CREATE TABLE `guestbook`"
cat db-init-demo/db_schema.sql | grep "INSERT INTO `guestbook`"
cat db-init-demo/db_schema.sql | grep "CREATE TABLE `hitLog`"
cat db-init-demo/db_schema.sql | grep "INSERT INTO `hitLog`"