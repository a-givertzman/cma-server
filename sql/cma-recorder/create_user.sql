DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_user WHERE usename = 'crane_data_server') THEN
        CREATE USER "crane_data_server" WITH PASSWORD '00d0-25e4-*&s2-ccds' CREATEDB CREATEROLE;
    END IF;
END
$$;