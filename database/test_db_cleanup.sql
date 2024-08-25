SELECT 'DROP DATABASE "' || datname || '";'
FROM pg_database
WHERE datistemplate = false
  AND datname not in ('rust-bc', 'postgres', 'test') \gexec
