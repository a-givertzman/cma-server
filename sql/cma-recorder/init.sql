SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;
SET default_tablespace = '';
SET default_table_access_method = heap;

CREATE TYPE public.user_role_enum AS ENUM('admin','operator');
ALTER TYPE public.user_role_enum OWNER TO crane_data_server;


CREATE TABLE public.app_user (
    id bigint NOT NULL,
    role public.user_role_enum NOT NULL,
    name character varying(255) NOT NULL,
    login character varying(255) NOT NULL,
    pass character varying(2584) NOT NULL,
    created timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted timestamp without time zone
);
ALTER TABLE public.app_user OWNER TO crane_data_server;
COMMENT ON TABLE public.app_user IS 'Пользователи';
COMMENT ON COLUMN public.app_user.role IS 'Признак группировки';
CREATE SEQUENCE public.app_user_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER TABLE public.app_user_id_seq OWNER TO crane_data_server;
ALTER SEQUENCE public.app_user_id_seq OWNED BY public.app_user.id;


CREATE TABLE public.event (
    uid bigint NOT NULL,
    "timestamp" timestamp without time zone NOT NULL,
    pid smallint NOT NULL,
    value smallint NOT NULL,
    status smallint NOT NULL
);
ALTER TABLE public.event OWNER TO crane_data_server;
COMMENT ON TABLE public.event IS 'События. Только тэги дискретных значений, регистрируется 0 - как норма, > 0 - как авария.';
CREATE SEQUENCE public.event_uid_seq1
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER TABLE public.event_uid_seq1 OWNER TO crane_data_server;
ALTER SEQUENCE public.event_uid_seq1 OWNED BY public.event.uid;


CREATE TABLE public.event_utils (
    id integer NOT NULL,
    row_count bigint,
    row_limit bigint,
    purge_batch_size integer,
    purge_shift_size integer,
    is_purge_running boolean
);
ALTER TABLE public.event_utils OWNER TO crane_data_server;
CREATE SEQUENCE public.event_utils_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER TABLE public.event_utils_id_seq OWNER TO crane_data_server;
ALTER SEQUENCE public.event_utils_id_seq OWNED BY public.event_utils.id;


CREATE TYPE public.tag_type_enum AS ENUM('Bool','Int','UInt','DInt','Word','LInt','Real','Time','Date_And_Time');
ALTER TYPE public.tag_type_enum OWNER TO crane_data_server;


CREATE TABLE public.tags (
    id integer NOT NULL,
    type public.tag_type_enum NOT NULL,
    name character varying(255) NOT NULL,
    description character varying(255) DEFAULT ''::character varying NOT NULL
);
ALTER TABLE public.tags OWNER TO crane_data_server;
COMMENT ON TABLE public.tags IS 'Справочник тэгов проекта.';
CREATE SEQUENCE public.tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER TABLE public.tags_id_seq OWNER TO crane_data_server;
ALTER SEQUENCE public.tags_id_seq OWNED BY public.tags.id;


CREATE VIEW public.event_view AS
    SELECT
        e.uid AS uid,
        e.pid AS pid,
        e.value AS value,
        e.status AS status,
        e.timestamp AS timestamp,
        t.type AS type,
        t.name AS name,
        t.description AS description
    FROM
        (public.event e
    LEFT JOIN public.tags t ON
        (e.pid = t.id));
ALTER VIEW public.event_view OWNER TO crane_data_server;


CREATE TABLE public.report (
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    code smallint NOT NULL,
    message character varying(50) NOT NULL,
    stack text NOT NULL
);
ALTER TABLE public.report OWNER TO crane_data_server;
COMMENT ON TABLE public.report IS 'Отчеты об ошибках в работе приложений.';

ALTER TABLE ONLY public.app_user ALTER COLUMN id SET DEFAULT nextval('public.app_user_id_seq'::regclass);
ALTER TABLE ONLY public.event ALTER COLUMN uid SET DEFAULT nextval('public.event_uid_seq1'::regclass);
ALTER TABLE ONLY public.event_utils ALTER COLUMN id SET DEFAULT nextval('public.event_utils_id_seq'::regclass);
ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);

ALTER TABLE ONLY public.app_user
    ADD CONSTRAINT app_user_login_key UNIQUE (login);
ALTER TABLE ONLY public.app_user
    ADD CONSTRAINT app_user_pkey PRIMARY KEY (id, login);
ALTER TABLE ONLY public.event
    ADD CONSTRAINT event_pkey PRIMARY KEY (uid);
ALTER TABLE ONLY public.event_utils
    ADD CONSTRAINT event_utils_pkey PRIMARY KEY (id);


ALTER TABLE ONLY public.report
    ADD CONSTRAINT report_pkey PRIMARY KEY ("timestamp");
ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_name_key UNIQUE (name);
ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);

CREATE INDEX idx_event_timestamp ON public.event USING btree ("timestamp");
CREATE INDEX idx_rec_operating_event_timestamp ON public.event USING btree ("timestamp");

CREATE OR REPLACE FUNCTION public.event_purge_records()
RETURNS void 
LANGUAGE plpgsql AS $$
DECLARE
    deleted INT;
    to_delete INT;
    batch_size INT;
    is_purge_possible BOOLEAN;
BEGIN
    SELECT (row_count - row_limit + purge_shift_size), purge_batch_size
    FROM event_utils INTO to_delete, batch_size;
    
    WITH upd_result AS (
        UPDATE event_utils SET is_purge_running = true WHERE id = 1 AND is_purge_running = false
        RETURNING *
    ) SELECT count(*) = 1 FROM upd_result INTO is_purge_possible;

    IF is_purge_possible THEN
        deleted := 0;
    
        WHILE (to_delete - deleted) > batch_size LOOP
            WITH del_result AS (
                DELETE FROM event
                WHERE ctid IN (
                    SELECT ctid FROM event
                    ORDER BY timestamp, uid ASC
                    LIMIT batch_size
                ) RETURNING *
            ) SELECT (count(*) + deleted) FROM del_result into deleted;
        END LOOP;

        DELETE FROM event WHERE ctid IN (
            SELECT ctid FROM event
            ORDER BY timestamp, uid ASC
            LIMIT (to_delete - deleted)
        );

        UPDATE event_utils SET is_purge_running = false WHERE id = 1;
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION public.event_check_for_purge()
RETURNS void 
LANGUAGE plpgsql
AS $$
DECLARE
    is_purge_needed BOOLEAN;
BEGIN
    SELECT row_count > row_limit FROM event_utils WHERE id = 1 INTO is_purge_needed;
    IF is_purge_needed THEN
        PERFORM public.event_purge_records();
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION public.event_counter_inc()
RETURNS trigger 
LANGUAGE plpgsql
AS $$
DECLARE
add_count INT;
BEGIN
    SELECT count(*) FROM new_tbl INTO add_count;
    UPDATE event_utils SET row_count = COALESCE(row_count, 0) + add_count WHERE id = 1;
    PERFORM public.event_check_for_purge();
    RETURN new;
END;
$$;

CREATE OR REPLACE FUNCTION public.event_counter_dec()
RETURNS trigger 
LANGUAGE plpgsql
AS $$
DECLARE
    del_count INT;
BEGIN
    SELECT count(*) FROM old_tbl INTO del_count;
    UPDATE event_utils SET row_count = COALESCE(row_count, 0) - del_count WHERE id = 1;
    RETURN new;
END;
$$;

CREATE TRIGGER event_delete_trigger AFTER DELETE ON public.event REFERENCING OLD TABLE AS old_tbl FOR EACH STATEMENT EXECUTE FUNCTION public.event_counter_dec();
CREATE TRIGGER event_insert_trigger AFTER INSERT ON public.event REFERENCING NEW TABLE AS new_tbl FOR EACH STATEMENT EXECUTE FUNCTION public.event_counter_inc();

GRANT ALL ON DATABASE crane_data_server TO crane_data_server;