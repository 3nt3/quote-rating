#!/bin/sh

timestamp="$(date -Ins)"

dir=/home/ente/quote-rating-backups

mkdir -p $dir
docker exec -it quote-rating_db_1 pg_dump -U quotes | gzip > $dir/dump_$timestamp.sql.gz
