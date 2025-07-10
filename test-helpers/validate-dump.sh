# the dump did not occur if the dump file is:
# - empty,
# - has an identical timestamp to the backup file, or
# - is missing any of the INSERT or CREATE TABLE statements for the tables

# the directory to check for the backup database dump file
# can be config'd with a command line arg
REF_DIR=db-init

if [[ $1 = "demo" ]]
then
    REF_DIR=db-init-demo
fi

if [[ $(tail -n 3 db-dumps/db_schema_*.sql) = $(tail -n 3 "$REF_DIR/db_schema.sql.bkup") ]]
then 
    exit 1
fi
cat db-dumps/db_schema_*.sql | grep "CREATE TABLE \`guestbook\`"
cat db-dumps/db_schema_*.sql | grep "INSERT INTO \`guestbook\`"
cat db-dumps/db_schema_*.sql | grep "CREATE TABLE \`hitLog\`"
cat db-dumps/db_schema_*.sql | grep "INSERT INTO \`hitLog\`"