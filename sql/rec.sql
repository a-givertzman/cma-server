
-- DROP TABLE public.rec_basic_metric;
CREATE TABLE public.rec_basic_metric (
	id bpchar(32) NOT NULL,
	"type" public."metric_data_type_enum" NOT NULL,
	"name" varchar(255) NOT NULL,
	description varchar(255) NOT NULL,
	value text NOT NULL,
	CONSTRAINT operating_metric_id_key UNIQUE (id),
	CONSTRAINT operating_metric_pkey PRIMARY KEY (name)
);

-- DROP TABLE public.rec_operating_cycle;
CREATE TABLE public.rec_operating_cycle (
	id int8 DEFAULT nextval('operating_cycle_id_seq'::regclass) NOT NULL,
	timestamp_start timestamp NOT NULL,
	timestamp_stop timestamp NOT NULL,
	alarm_class bpchar(2) NOT NULL,
	CONSTRAINT operating_cycle_pkey PRIMARY KEY (id)
);

-- drop table public.rec_event_name;
CREATE TABLE public.rec_event_name (
	id serial PRIMARY KEY,
	"name" varchar(255) NOT NULL,
	description varchar(255) DEFAULT ''::character varying NOT NULL,
	CONSTRAINT rec_event_name_name UNIQUE (name)
);
-- drop index idx_rec_operating_event_timestamp;
-- drop table public.rec_operating_event;
CREATE TABLE public.rec_operating_event (
	id bigserial PRIMARY KEY,
	operating_cycle_id int8 NOT NULL,
	"timestamp" timestamp NOT NULL,
	name_id int4 NOT NULL,
	value numeric(16, 8) NOT NULL,
	status int2 NOT NULL
);
CREATE INDEX idx_rec_operating_event_timestamp ON public.event USING btree ("timestamp");

-- DROP TABLE public.rec_operating_metric;
CREATE TABLE public.rec_operating_metric (
	operating_cycle_id int8 NOT NULL,
	pid int4 NOT NULL,
	metric_id bpchar(16) NOT NULL,
	value numeric(16, 8) NOT NULL,
	CONSTRAINT operating_cycle_metric_value_pkey PRIMARY KEY (operating_cycle_id, pid, metric_id)
);

-- DROP TABLE public.rec_operating_metric_dict;
CREATE TABLE public.rec_operating_metric_dict (
	id bpchar(16) NOT NULL,
	"name" varchar(255) NOT NULL,
	description varchar(255) NOT NULL,
	CONSTRAINT operating_cycle_metric_pkey PRIMARY KEY (id)
);

-- drop view public.operating_metric_view
CREATE OR REPLACE VIEW public.operating_metric_view
AS SELECT ocmv.operating_cycle_id,
--    ocmv.pid AS point_id,
--    tag.name AS point_name,
    ocmv.metric_id,
    ocm.name AS metric_name,
    ocm.description AS metric_description,
    ocmv.value
   FROM rec_operating_metric ocmv
--     LEFT JOIN tags tag ON ocmv.pid = tag.id
     LEFT JOIN rec_operating_metric_dict ocm ON ocmv.metric_id = ocm.id;
