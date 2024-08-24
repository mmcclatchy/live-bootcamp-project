-- database/test_db_cleanup.sql

-- Generate DROP DATABASE commands
\t
\o /tmp/drop_commands.sql
SELECT format('DROP DATABASE IF EXISTS %I;', datname)
FROM pg_database
WHERE datistemplate = false AND datname not in ('rust-bc', 'postgres');
\o
\t

-- Execute the generated commands
\i /tmp/drop_commands.sql

-- Clean up the temporary file
\! rm /tmp/drop_commands.sql
