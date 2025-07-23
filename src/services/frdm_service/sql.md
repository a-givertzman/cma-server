# FRDM (Rope Defects Monitoring)

## frdm_settings

key                    |  value
---------------------- | -------
rope_length            |  3000  (m)
defect_slices          |  30000
deprication_slices     |  60000

```sql
-- FRDM | Setting parameters
create table public.frdm_settings (
    id                  varchar primary key not null,
    value               text not null
);
```
---

## frdm_defect

id  |  defect  |  timestamp  | count | acknowledged | deleted

```sql
-- FRDM | Defects type
-- Enum of geometry defect type`s
-- containing the position of defect withing a frame';
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
-- FRDM | Defects
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
```
---

## frdm_defect_image

id | frdm_defect_id | camera_id | path

```sql
-- FRDM | Images of the rope defects
create table public.frdm_defect_image (
    id                  bigserial not null,
    frdm_defect_id      int8 not null,
    camera_id           int2 not null,
    path                text not null,
    created             timestamp default current_timestamp not null,
    PRIMARY KEY (id, frdm_defect_id, camera_id)
);

-- Insert or update defect and associated image
do $$
begin
	insert into public.frdm_defect (id, defect, first, last, score)
	    values (3, 'expansion', current_timestamp, current_timestamp, 1)
	on conflict (id, defect) do update 
	    set (last, score) = (current_timestamp, frdm_defect.score + 1);
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 1, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 2, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 3, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 4, 'assets/frdm/defect_image/1.jpeg');
	EXCEPTION
		WHEN others then
			rollback;
end; $$
language plpgsql;

-- FRDM | Function cleaning the old images keeping 10 imeges per rope slice for each defect tipe
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
end; $$ 
language plpgsql;

-- FRDM | Trigger for `frdm_defect_image` table to call cleaning oafter each insert
create trigger clean_frdm_defect_image
    after insert on public.frdm_defect_image
    for each row
    execute procedure clean_frdm_defect_image();
```

## frdm_deprecation

id  |  deprecation

```sql
-- FRDM | Rope deprecation values
-- Rope devided for slices, deprecation value calculated for each slice
-- sliceLingth = ropeLength / slices'
create table public.frdm_deprecation (
    id                  bigserial primary key not null,
    deprecation         double precision default 0.0 not null	
);
