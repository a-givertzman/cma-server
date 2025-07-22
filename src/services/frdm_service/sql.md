# FRDM (Rope Defects Monitoring)

## frdm_settings

key                    |  value
---------------------- | -------
rope_length            |  3000  (m)
defect_slices          |  30000
deprication_slices     |  60000

```sql
comment on type public.frdm_settings is E''
    'FRDM (Fiber Rope Defects Monitoring) setting parameters'
create table public.frdm_settings (
    id                  varchar primary key not null,
    value               text not null,
);
```
---

## frdm_defect

id  |  defect  |  timestamp  | count | acknowledged | deleted

```sql
comment on type public.frdm_defect_enum is E''
    'Enum of geometry defect type`s'
    'containing the position of defect withing a frame';

create type public.frdm_defect_enum as enum (
    -- Detecting both sides width growing
    'expansion',
    -- Detecting both sides width reduction
    'compressing',
    -- Detecting one side raising
    'hill',
    -- Detecting one side drooping
    'pit',
);

comment on type public.frdm_defect is E''
    'FRDM (Fiber Rope Defects Monitoring) defects'
create table public.frdm_defect (
    id                  bigint primary key not null,
    defect              frdm_defect_enum not null,
    first               timestamp not null,
    last                timestamp not null
    count               int8 default 0 not null,
    acknowledged        timestamp null,
    deleted             timestamp null,
);
```
---

## frdm_defect_image

id | frdm_defect_id | camera_id | path

```sql
comment on type public.frdm_defect is E''
    'FRDM images '
create table public.frdm_defect_image (
    id                  bigserial primary key not null,
    frdm_defect_id      int8 not null,
    camera_id           int2 not null,
    path                text not null,
);
create trigger clean_frdm_defect_image
    after insert on public.frdm_defect_image
    for each row
    execute procedure clean_frdm_defect_image();
```

## frdm_deprecation

id  |  deprecation

```sql
comment on type public.frdm_defect is E''
    'FRDM (Fiber Rope Defects Monitoring) deprecation values'
    'Rope devided for slices, deprecation value calculated for each slice'
    'sliceLingth = ropeLength / slices'
create table public.frdm_deprecation (
    id                  bigserial primary key not null,
    deprecation         double default 0.0 not null,
);
```
