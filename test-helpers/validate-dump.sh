if [  $(tail -n 1 ../db-init-demo/db_schema.sql) \
    = $(tail -n 1 ../db-init-demo/db_schema.sql.bkup) ] 
then 
    exit 1
fi
cat ../db-init-demo/db_schema.sql | grep "CREATE TABLE `guestbook`"
cat ../db-init-demo/db_schema.sql | grep "INSERT INTO `guestbook`"
cat ../db-init-demo/db_schema.sql | grep "CREATE TABLE `hitLog`"
cat ../db-init-demo/db_schema.sql | grep "INSERT INTO `hitLog`"