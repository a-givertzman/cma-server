# FRDM (Rope Defects Monitoring)

## frdm_settings

key                    |  value
---------------------- | -------
rope_length            |  3000  (m)
defect_slices          |  30000
deprication_slices     |  60000

```sql
create table public.frdm_settings (
    id                  varchar primary key not null,
    value               text not null
);
comment on table public.frdm_settings is 'FRDM (Fiber Rope Defects Monitoring) setting parameters';
```
---

## frdm_defect

id  |  defect  |  timestamp  | count | acknowledged | deleted

```sql
create type public.frdm_defect_enum as enum (
    -- Detecting both sides width growing
    'expansion',
    -- Detecting both sides width reduction
    'compressing',
    -- Detecting one side raising
    'hill',
    -- Detecting one side drooping
    'pit'
);
comment on type public.frdm_defect_enum is '
    Enum of geometry defect type`s
    containing the position of defect withing a frame';


create table public.frdm_defect (
    id                  bigint not null,
    defect              frdm_defect_enum not null,
    camera              int2 not null,
    first               timestamp not null,
    last                timestamp not null,
    score               int8 default 0 not null,
    acknowledged        timestamp null,
    deleted             timestamp null,
    PRIMARY KEY (id, defect, camera)
);
comment on table public.frdm_defect is 'FRDM (Fiber Rope Defects Monitoring) defects';
```
---

## frdm_defect_image

id | frdm_defect_id | camera_id | path

```sql
create table public.frdm_defect_image (
    id                  bigserial not null,
    frdm_defect_id      int8 not null,
    camera_id           int2 not null,
    path                text not null,
    created             timestamp default current_timestamp not null,
    PRIMARY KEY (id, frdm_defect_id, camera_id)
);
comment on table public.frdm_defect_image is 'FRDM images ';

create or replace function clean_frdm_defect_image() returns trigger as $$
begin
    delete from public.frdm_defect_image
        where (id) in (
            select id
            from public.frdm_defect_image
            order by created
            limit 1
        );
   return new;
end;
$$ language plpgsql;
create trigger clean_frdm_defect_image
    after insert on public.frdm_defect_image
    for each row
    execute procedure clean_frdm_defect_image();
```

## frdm_deprecation

id  |  deprecation

```sql
create table public.frdm_deprecation (
    id                  bigserial primary key not null,
    deprecation         double precision default 0.0 not null	
);
comment on table public.frdm_deprecation is '
    FRDM (Fiber Rope Defects Monitoring) deprecation values
    Rope devided for slices, deprecation value calculated for each slice
    sliceLingth = ropeLength / slices'
```
