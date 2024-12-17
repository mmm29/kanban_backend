# Seed the database.
psql -d ${DATABASE} -f ../postgresdb/init-schema.sql
psql -d ${DATABASE} -f seeds.sql