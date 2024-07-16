
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

CREATE TABLE public."rec_operating_event" (
	uid bigserial PRIMARY KEY,
	"timestamp" timestamp NOT NULL,
	pid int2 NOT NULL,
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